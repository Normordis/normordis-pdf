use serde::{Deserialize, Serialize};

// ── NDT 2.0.0 Root ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NdtDocument {
    pub ndt_version: String,
    pub schema_id: String,
    pub versao_ndt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub titulo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referencia_legal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estilos: Option<Estilos>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutGlobal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recursos: Vec<Recurso>,
    pub paginas_def: Vec<PaginaDef>,
    pub sequencia: Vec<SequenciaEntrada>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub composicao: Vec<ComposicaoEntrada>,
}

// ── Estilos ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Estilos {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fonte_padrao: Option<Fonte>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cor_texto: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cor_primaria: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub espacamento_entre_paragrafos_mm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identacao_lista_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cabecalhos: Vec<CabecalhoEstilo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CabecalhoEstilo {
    pub nivel: u8,
    pub fonte: Fonte,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Fonte {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub familia: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tamanho: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peso: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estilo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cor: Option<String>,
}

// ── Layout ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutGlobal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formato: Option<FormatoFormato>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientacao: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margens: Option<Margens>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FormatoFormato {
    Named(String),
    Custom { largura: f64, altura: f64 },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Margens {
    pub topo: f64,
    pub fundo: f64,
    pub esq: f64,
    pub dir: f64,
}

// ── Primitivos ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Posicao {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Contorno {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub espessura: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cor: Option<String>,
}

// ── Recursos ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "modo", rename_all = "snake_case")]
pub enum Recurso {
    Embebido {
        id: String,
        tipo: String,
        dados: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        familia: Option<String>,
    },
    ReferenciadoPorHash {
        id: String,
        tipo: String,
        hash_sha256: String,
        content_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        familia: Option<String>,
    },
}

// ── Gráficos ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Grafico {
    Linha {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        de: Posicao,
        para: Posicao,
        #[serde(skip_serializing_if = "Option::is_none")]
        espessura: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cor: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        estilo: Option<String>,
    },
    Rectangulo {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        posicao: Posicao,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        preenchimento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        contorno: Option<Contorno>,
        #[serde(skip_serializing_if = "Option::is_none")]
        raio_canto: Option<f64>,
    },
    GrelhaDigitos {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        referencia: String,
        posicao: Posicao,
        num_caixas: u32,
        largura_caixa: f64,
        altura_caixa: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        espacamento: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cor_contorno: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        espessura_contorno: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotulo_acessivel: Option<String>,
    },
    Imagem {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        referencia_recurso: String,
        posicao: Posicao,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        manter_proporcao: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
    },
    TextoFixo {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        conteudo: String,
        posicao: Posicao,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
    },
    Assinatura {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotulo: Option<String>,
        posicao: Posicao,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        modo: Option<String>,
    },
    CodigoBarras {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        formato_barras: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        referencia: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        conteudo: Option<String>,
        posicao: Posicao,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        nivel_correcao: Option<String>,
    },
    Poligono {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        pontos: Vec<Posicao>,
        #[serde(skip_serializing_if = "Option::is_none")]
        preenchimento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        contorno: Option<Contorno>,
    },
    Elipse {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        centro: Posicao,
        raio_x: f64,
        raio_y: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        preenchimento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        contorno: Option<Contorno>,
    },
    Svg {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        referencia_recurso: String,
        posicao: Posicao,
        largura: f64,
        altura: f64,
    },
    TabelaVisual {
        #[serde(skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotacao: Option<f64>,
        posicao: Posicao,
        largura: f64,
        altura_linha: f64,
        num_linhas: u32,
        colunas: Vec<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        contorno: Option<Contorno>,
    },
}

// ── Campos ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Campo {
    pub referencia: String,
    pub posicao: Posicao,
    pub largura: f64,
    pub altura: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formato: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub casas_decimais: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fonte: Option<Fonte>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alinhamento: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preenchimento_fundo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotulo_acessivel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incluir_se: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descontinuado: Option<bool>,
}

// ── Blocos ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColunaTabela {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cabecalho: Option<String>,
    pub largura: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alinhamento: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formato: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub casas_decimais: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descontinuado: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EstiloCabecalhoTabela {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fundo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fonte: Option<Fonte>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Bloco {
    Tabela {
        referencia: String,
        posicao: Posicao,
        largura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        altura_linha: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_linhas_visivel: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        repete_cabecalho: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        estilo_cabecalho: Option<EstiloCabecalhoTabela>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
        colunas: Vec<ColunaTabela>,
    },
    Corpo {
        referencia: String,
        posicao: Posicao,
        largura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte_base: Option<Fonte>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Cabecalho {
        referencia: String,
        posicao: Posicao,
        largura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Rodape {
        referencia: String,
        posicao: Posicao,
        largura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
}

// ── Fluxo ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fluxo {
    pub y_inicio: f64,
    pub elementos: Vec<ElementoFluxo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LateralColuna {
    pub largura: f64,
    pub conteudo: Vec<ElementoFluxoLinhLateralConteudo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum ElementoFluxoLinhLateralConteudo {
    Tabela {
        referencia: String,
        largura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        altura_linha: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_linhas_visivel: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        repete_cabecalho: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        estilo_cabecalho: Option<EstiloCabecalhoTabela>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
        colunas: Vec<ColunaTabela>,
    },
    TextoFixo {
        conteudo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Campo {
        referencia: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        formato: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        casas_decimais: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotulo_acessivel: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Imagem {
        referencia_recurso: String,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        manter_proporcao: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Espaco {
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Separador {
        #[serde(skip_serializing_if = "Option::is_none")]
        espessura: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cor: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Assinatura {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotulo: Option<String>,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        modo: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum ElementoFluxo {
    Corpo {
        referencia: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Tabela {
        referencia: String,
        largura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        altura_linha: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_linhas_visivel: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        repete_cabecalho: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        estilo_cabecalho: Option<EstiloCabecalhoTabela>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
        colunas: Vec<ColunaTabela>,
    },
    TextoFixo {
        conteudo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Campo {
        referencia: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        formato: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        casas_decimais: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotulo_acessivel: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Imagem {
        referencia_recurso: String,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        manter_proporcao: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Espaco {
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Separador {
        #[serde(skip_serializing_if = "Option::is_none")]
        espessura: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cor: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    Assinatura {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        rotulo: Option<String>,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        modo: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    LinhaLateral {
        elementos: Vec<LateralColuna>,
        #[serde(skip_serializing_if = "Option::is_none")]
        incluir_se: Option<String>,
    },
    QuebraPagina,
}

// ── Mobília ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum MobiliaItem {
    NumeroPagina {
        formato: String,
        posicao: Posicao,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
    },
    TextoFixo {
        conteudo: String,
        posicao: Posicao,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
    },
    CampoNdf {
        referencia: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        formato: Option<String>,
        posicao: Posicao,
        largura: f64,
        altura: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        alinhamento: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
    },
    MarcaAgua {
        conteudo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        opacidade: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        angulo: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fonte: Option<Fonte>,
    },
}

// ── PaginaDef ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginaDef {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formato: Option<FormatoFormato>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margens: Option<Margens>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graficos: Vec<Grafico>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub campos: Vec<Campo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocos: Vec<Bloco>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fluxo: Option<Fluxo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mobilia: Vec<MobiliaItem>,
}

// ── Sequência ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SequenciaEntrada {
    pub pagina_def: String,
    pub repeticao: Repeticao,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fonte_overflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linhas_por_pagina: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incluir_se: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeticao {
    Unica,
    PorLinha,
    ConformeNecessario,
}

// ── Composição ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComposicaoEntrada {
    pub id: String,
    pub schema_id: String,
    pub resolver: ComposicaoResolver,
    pub posicao: ComposicaoPosicao,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apos_bloco: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obrigatorio: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComposicaoResolver {
    pub tipo: String,
    pub template: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComposicaoPosicao {
    Antes,
    Apos,
    AposBloco,
}

// ── Legacy output / signature (exported, not part of NdtDocument root) ────────

/// Output-level options (kept for library consumers that reference this type).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NdtOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility: Option<crate::compliance::ua::AccessibilityConfig>,
}

/// Signature metadata (kept for library consumers that reference this type).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NdtSignature {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<NdtSignatureField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Visual position of a signature field on the page.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NdtSignatureField {
    pub x_mm: f64,
    pub y_mm: f64,
    pub width_mm: f64,
    pub height_mm: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

// ── Legacy body-rendering types (pub(crate) — NDF pipeline only) ─────────────

pub(crate) mod legacy_body {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum BodyElement {
        Paragraph(ParagraphElement),
        Heading(HeadingElement),
        RichText(RichTextElement),
        Table(TableElement),
        List(ListElement),
        Image(ImageElement),
        Spacer(SpacerElement),
        HorizontalRule,
        PageBreak,
        FixedText(FixedTextElement),
        FixedImage(FixedImageElement),
        FixedLine(FixedLineElement),
        FixedBox(FixedBoxElement),
        FootnoteRef(FootnoteRefElement),
        Toc(TocElement),
        AcroformField(AcroformFieldElement),
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct FootnoteRefElement {
        pub number: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mark_style: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TocElement {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_level: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub leader_char: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct AcroformFieldElement {
        pub field_type: String,
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tooltip: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub required: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_length: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub checked_by_default: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub options: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub font_size: Option<f64>,
        pub rect: AcroformRect,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct AcroformRect {
        pub x_mm: f64,
        pub y_mm: f64,
        pub width_mm: f64,
        pub height_mm: f64,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ParagraphElement {
        pub text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub alignment: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub font_size: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bold: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub italic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub indent_mm: Option<f64>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct HeadingElement {
        pub text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub level: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub alignment: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct RichTextElement {
        pub content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub source: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TableElement {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub headers: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rows: Option<Vec<Vec<String>>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub col_widths: Option<Vec<f64>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stripe: Option<bool>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ListElement {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub list_type: Option<String>,
        pub items: Vec<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ImageElement {
        pub src: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub width_percent: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub alignment: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub caption: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SpacerElement {
        pub height_mm: f64,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct FixedTextElement {
        pub x_mm: f64,
        pub y_mm: f64,
        pub width_mm: f64,
        pub height_mm: f64,
        pub text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub alignment: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub font_size: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub overflow: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub padding_mm: Option<f64>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct FixedImageElement {
        pub x_mm: f64,
        pub y_mm: f64,
        pub width_mm: f64,
        pub height_mm: f64,
        pub src: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub fit: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct FixedLineElement {
        pub x1_mm: f64,
        pub y1_mm: f64,
        pub x2_mm: f64,
        pub y2_mm: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub width_mm: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub color: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct FixedBoxElement {
        pub x_mm: f64,
        pub y_mm: f64,
        pub width_mm: f64,
        pub height_mm: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub alignment: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub overflow: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub padding_mm: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub border_color: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub border_width_mm: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub background: Option<String>,
    }
}
