use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use crate::{DocumentBuilder, NormaxisPdfError};
use crate::template::{parse_ndt, parse_ndt_data, render as render_ndt_template};
use crate::styles::DocumentStyle;

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
#[unsafe(no_mangle)]
pub extern "C" fn generate_pdf_from_json(json_config: *const c_char) -> *mut PdfResult {
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

/// Libera a memória alocada por `generate_pdf_from_json`.
#[unsafe(no_mangle)]
pub extern "C" fn free_pdf_result(result: *mut PdfResult) {
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
#[unsafe(no_mangle)]
pub extern "C" fn generate_pdf_from_ndt(
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

fn create_pdf_from_ndt(ndt_json: &str, data_json: &str) -> Result<Vec<u8>, NormaxisPdfError> {
    let doc = parse_ndt(ndt_json)
        .map_err(|e| NormaxisPdfError::Template(e.to_string()))?;
    let data = parse_ndt_data(data_json)
        .map_err(|e| NormaxisPdfError::Template(e.to_string()))?;

    let title = doc.meta.as_ref().and_then(|m| m.title.as_deref()).unwrap_or("Document");
    let style = DocumentStyle::default();
    let elements = render_ndt_template(&doc, &data, &style)
        .map_err(|e| NormaxisPdfError::Template(e.to_string()))?;

    let mut builder = DocumentBuilder::new(title);
    for el in elements {
        builder = builder.push_boxed(el);
    }
    builder.render_to_bytes()
}

// Função interna para criar o PDF (adapta ao teu código real)
fn create_pdf_from_json(json: &str) -> Result<Vec<u8>, NormaxisPdfError> {
    // Parsing básico do JSON
    let config: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| NormaxisPdfError::ParseError(e.to_string()))?;

    let title = config.get("title").and_then(|v| v.as_str()).unwrap_or("Documento");
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
                            let level = elem.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
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