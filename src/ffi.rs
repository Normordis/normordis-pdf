use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use crate::template::parse_ndt;
use crate::{DocumentBuilder, NormordisPdfError};

/// Gera um PDF a partir de um JSON de configuração.
/// Exemplo de JSON:
/// {
///   "title": "Documento Teste",
///   "elements": [
///     {"type": "paragraph", "text": "Olá, mundo!", "align": "left"}
///   ]
/// }
/// Retorna um ponteiro para os bytes do PDF (alocado com malloc).
/// O chamador deve liberar com `free_pdf_result`.
///
/// # Safety
/// `json_config` deve ser nulo ou apontar para uma C-string válida,
/// terminada em NUL, legível durante a chamada (contrato normal de
/// `CStr::from_ptr`). O ponteiro devolvido, quando não nulo, só deve ser
/// libertado com `free_pdf_result` — nunca com `free()`/`delete` do lado
/// chamador, nem mais do que uma vez.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate_pdf_from_json(json_config: *const c_char) -> *mut PdfResult {
    if json_config.is_null() {
        return ptr::null_mut();
    }

    let json_str = unsafe { CStr::from_ptr(json_config).to_string_lossy() };

    // Parse do JSON (simplificado; usa serde para parsing real)
    // Aqui, assumimos um JSON básico. Para templates NDT, ajusta conforme o teu código.
    let pdf_bytes = match create_pdf_from_json(&json_str) {
        Ok(bytes) => bytes,
        Err(_) => return ptr::null_mut(),
    };

    // Aloca memória para o resultado
    let result = Box::new(PdfResult {
        data: pdf_bytes.as_ptr() as *mut u8,
        len: pdf_bytes.len(),
        _owned: pdf_bytes, // Mantém os dados vivos
    });

    Box::into_raw(result)
}

/// Liberta a memória alocada por `generate_pdf_from_json` ou
/// `generate_pdf_from_ndt`.
///
/// # Safety
/// `result` deve ser nulo ou um ponteiro devolvido por uma dessas duas
/// funções desta mesma crate, ainda não libertado. Chamar duas vezes com
/// o mesmo ponteiro (double free), ou passar um ponteiro de outra
/// origem, é comportamento indefinido. O ponteiro não deve ser usado
/// depois desta chamada.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_pdf_result(result: *mut PdfResult) {
    if !result.is_null() {
        unsafe { drop(Box::from_raw(result)) };
    }
}

/// Estrutura para retornar dados do PDF (bytes + tamanho).
#[repr(C)]
pub struct PdfResult {
    pub data: *mut u8,
    pub len: usize,
    _owned: Vec<u8>, // Campo privado para gerenciar memória
}

/// Generate a PDF from an NDT template JSON + NdtData JSON.
///
/// `ndt_json` — UTF-8 NDT template (JSON or TOML).
/// `data_json` — UTF-8 NdtData JSON (`{"ndt_data":"1.0.0","data":{...}}`).
///
/// Returns a `PdfResult` pointer on success, or null on error.
/// The caller must free with `free_pdf_result`.
///
/// # Safety
/// `ndt_json` e `data_json` devem ser nulos ou apontar cada um para uma
/// C-string válida, terminada em NUL, legível durante a chamada. O
/// ponteiro devolvido, quando não nulo, só deve ser libertado com
/// `free_pdf_result`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate_pdf_from_ndt(
    ndt_json: *const c_char,
    data_json: *const c_char,
) -> *mut PdfResult {
    if ndt_json.is_null() || data_json.is_null() {
        return ptr::null_mut();
    }
    let ndt_str = unsafe { CStr::from_ptr(ndt_json).to_string_lossy() };
    let data_str = unsafe { CStr::from_ptr(data_json).to_string_lossy() };

    let pdf_bytes = match create_pdf_from_ndt(&ndt_str, &data_str) {
        Ok(bytes) => bytes,
        Err(_) => return ptr::null_mut(),
    };

    let result = Box::new(PdfResult {
        data: pdf_bytes.as_ptr() as *mut u8,
        len: pdf_bytes.len(),
        _owned: pdf_bytes,
    });
    Box::into_raw(result)
}

fn create_pdf_from_ndt(ndt_json: &str, data_json: &str) -> Result<Vec<u8>, NormordisPdfError> {
    // Só para extrair o título antes de construir o builder — o parsing e a
    // renderização propriamente ditos ficam a cargo de `push_ndt`, a mesma
    // API pública usada por quem consome a crate em Rust (evita duplicar,
    // e desalinhar, o pipeline NDT aqui).
    let doc = parse_ndt(ndt_json).map_err(|e| NormordisPdfError::Template(e.to_string()))?;
    let title = doc.titulo.as_deref().unwrap_or("Document");

    DocumentBuilder::new(title)
        .push_ndt(ndt_json, data_json)?
        .render_to_bytes()
}

// Função interna para criar o PDF (adapta ao teu código real)
fn create_pdf_from_json(json: &str) -> Result<Vec<u8>, NormordisPdfError> {
    // Parsing básico do JSON
    let config: serde_json::Value =
        serde_json::from_str(json).map_err(|e| NormordisPdfError::ParseError(e.to_string()))?;

    let title = config
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Documento");
    let mut builder = DocumentBuilder::new(title);

    if let Some(elements) = config.get("elements").and_then(|v| v.as_array()) {
        for elem in elements {
            if let Some(elem_type) = elem.get("type").and_then(|v| v.as_str()) {
                match elem_type {
                    "paragraph" => {
                        if let Some(text) = elem.get("text").and_then(|v| v.as_str()) {
                            let mut para = crate::Paragraph::new(text);
                            if let Some(align) = elem.get("align").and_then(|v| v.as_str()) {
                                let text_align = match align {
                                    "left" => crate::TextAlign::Left,
                                    "center" => crate::TextAlign::Center,
                                    "right" => crate::TextAlign::Right,
                                    "justify" => crate::TextAlign::Justify,
                                    _ => crate::TextAlign::Left,
                                };
                                para = para.align(text_align);
                            }
                            builder = builder.push(para);
                        }
                    }
                    "section" => {
                        if let Some(title) = elem.get("title").and_then(|v| v.as_str()) {
                            let level =
                                elem.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
                            builder = builder.push(crate::Section::new(title, level));
                        }
                    }
                    "spacer" => {
                        let height = elem.get("height").and_then(|v| v.as_f64()).unwrap_or(12.0);
                        builder = builder.push(crate::Spacer::new(height));
                    }
                    _ => {} // Ignora tipos desconhecidos
                }
            }
        }
    }

    builder.render_to_bytes()
}
