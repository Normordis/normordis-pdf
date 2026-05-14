# normordis-pdf — Guia do Programador

## Índice

1. [Conceitos fundamentais](#1-conceitos-fundamentais)
2. [DocumentBuilder](#2-documentbuilder)
3. [Estilo e configuração](#3-estilo-e-configuração)
4. [Elementos de fluxo](#4-elementos-de-fluxo)
5. [Elementos fixos](#5-elementos-fixos)
6. [Formato NCRTF](#6-formato-ncrtf)
7. [Fontes](#7-fontes) — embebidas, `register_bytes`, `register_file`, `load_dir`, fallback chain
8. [Tratamento de erros](#8-tratamento-de-erros)
9. [Implementar um elemento personalizado](#9-implementar-um-elemento-personalizado)
10. [Sistema de coordenadas](#10-sistema-de-coordenadas)

---

## 1. Conceitos fundamentais

### Dois modos de layout

**Flow** — o cursor vertical avança de cima para baixo. Cada elemento empurra o seguinte para baixo. Quando o cursor ultrapassa a margem inferior, a página fecha automaticamente e o cabeçalho é re-injectado na nova página. Este é o comportamento por omissão.

**Fixed** — o elemento é colocado em coordenadas absolutas (`x_mm`, `y_mm` a partir do canto inferior esquerdo da página). O cursor de fluxo não é afectado. Útil para templates de formulários, zonas de assinatura e carimbos.

Os dois modos podem coexistir no mesmo documento: os elementos fixos são sobrepostos ao conteúdo de fluxo sem o perturbar.

### Pipeline de renderização

```
DocumentBuilder
    │
    ▼
Document::render_to_bytes()
    │
    ├─ para cada elemento:
    │   ├─ LayoutMode::Flow   → verifica overflow → element.render() → flow.advance()
    │   └─ LayoutMode::Fixed  → element.render()  (sem verificação, sem avanço)
    │
    ├─ element.render() empurra printpdf::Op para ctx.ops
    │
    └─ no fim de cada página: PdfPage::new(width, height, ops)
```

### Sistema de coordenadas

`printpdf` usa origem no canto inferior esquerdo. `y = 0` é o fundo da página; `y = altura_da_página` é o topo. Todas as medidas são em **milímetros** (`f64`).

O cursor de fluxo começa em `page_height - margin_top` e decresce à medida que o conteúdo é adicionado.

---

## 2. DocumentBuilder

A API principal da crate. Método de construção fluente — cada método devolve `Self`.

```rust
use normordis_pdf::{DocumentBuilder, DocumentStyle, Paragraph, Section, Spacer};

let bytes = DocumentBuilder::new("Título do Documento")
    .style(DocumentStyle::default())          // opcional — omitir usa A4 com defaults
    .push(Section::new("Introdução", 1))
    .push(Paragraph::new("Corpo do texto..."))
    .push(Spacer::new(5.0))
    .render_to_bytes()?;
```

### Referência dos métodos

| Método | Descrição |
|---|---|
| `new(title)` | Cria um builder com o título do PDF |
| `style(DocumentStyle)` | Define o estilo global (margens, tamanhos de fonte, cores) |
| `fonts(FontRegistry)` | Fornece um registo de fontes pré-carregadas |
| `font_from_bytes(name, regular, bold?, italic?, bold_italic?)` | Regista uma família de fontes a partir de bytes `&[u8]` |
| `font_from_file(name, regular, bold?, italic?, bold_italic?)` | Regista uma família de fontes a partir de ficheiros TTF/OTF |
| `fonts_from_dir(dir)` | Carrega todas as fontes TTF/OTF de um directório |
| `default_font(name)` | Define a família de fontes por omissão do documento |
| `header(InstitutionalHeader)` | Cabeçalho institucional re-injectado em cada página |
| `footer(PageFooter)` | Rodapé em cada página |
| `push(element)` | Adiciona qualquer elemento de fluxo ao corpo |
| `push_ncrtf(json)` | Parseia JSON NCRTF e adiciona todos os blocos resultantes |
| `push_ndt(template, data)` | Parseia um template NDT e adiciona todos os elementos resultantes |
| `fixed_text(box_def, content, alignment)` | Caixa de texto em posição fixa |
| `fixed_image(box_def, data, fit)` | Imagem em posição fixa |
| `fixed_line(x1, y1, x2, y2, color)` | Linha decorativa em posição fixa |
| `render_to_bytes()` | Renderiza e devolve `Vec<u8>` com o PDF |
| `render_to_file(path)` | Renderiza e escreve directamente para ficheiro |

---

## 3. Estilo e configuração

### `DocumentStyle`

```rust
use normordis_pdf::{DocumentStyle, PageSize, RgbColor};

let style = DocumentStyle {
    page_size: PageSize::A4,
    margin_top_mm: 20.0,
    margin_bottom_mm: 20.0,
    margin_left_mm: 25.0,
    margin_right_mm: 20.0,
    font_size_body: 11.0,    // pontos tipográficos
    font_size_title: 16.0,
    font_size_section: 13.0,
    font_size_small: 9.0,
    line_height: 1.4,        // multiplicador (1.4 = 140% do tamanho da fonte)
    primary_color: RgbColor::from_hex("#003399").unwrap(),  // azul institucional
    text_color: RgbColor::from_hex("#1A1A1A").unwrap(),
};
```

O `Default` usa A4, margens 20/25 mm, fonte corpo 11 pt, azul `#003399`.

### `PageSize`

```rust
PageSize::A4       // 210 × 297 mm
PageSize::A3       // 297 × 420 mm
PageSize::Letter   // 215.9 × 279.4 mm
```

### `RgbColor`

```rust
RgbColor::new(0.0, 0.2, 0.6)          // componentes 0.0–1.0
RgbColor::from_hex("#003399")          // devolve Option<RgbColor>
RgbColor::from_hex("003399")           // sem o '#' também funciona
```

### `TextAlignment`

NORMAXIS suporta exactamente três alinhamentos:

```rust
TextAlignment::Left     // padrão para corpo, listas, células de tabela
TextAlignment::Center   // cabeçalhos, legendas, cabeçalhos de tabela
TextAlignment::Justify  // texto de corpo em documentos formais (padrão de Paragraph)
```

---

## 4. Elementos de fluxo

Todos os elementos de fluxo implementam o trait `Element` e são adicionados via `builder.push(elemento)`.

### `Paragraph`

```rust
use normordis_pdf::{Paragraph, TextAlignment};

// Texto simples
Paragraph::new("Texto do parágrafo.")

// Com alinhamento, bold, tamanho de fonte
Paragraph::new("Importante")
    .bold()
    .align(TextAlignment::Center)
    .font_size(14.0)

// Com runs formatados (produzido pelo conversor NCRTF, mas construível manualmente)
use normordis_pdf::{ParagraphContent, TextRun};
use normordis_pdf::richtext::marks::AppliedStyle;

Paragraph::from_runs(
    vec![
        TextRun { text: "Normal ".into(), style: AppliedStyle::default() },
        TextRun { text: "negrito".into(), style: AppliedStyle { bold: true, ..Default::default() } },
    ],
    TextAlignment::Left,
    None,  // font_size (None = usa DocumentStyle.font_size_body)
)
```

**Métodos de construção:**

| Método | Descrição |
|---|---|
| `Paragraph::new(text)` | Parágrafo de texto simples com Justify |
| `Paragraph::from_runs(runs, alignment, font_size)` | A partir de runs formatados |
| `.bold()` | Activa negrito (só para `Plain`) |
| `.italic()` | Activa itálico (só para `Plain`) |
| `.align(TextAlign)` | Define o alinhamento |
| `.font_size(f64)` | Tamanho de fonte em pontos |
| `.font_family(name)` | Substitui a família de fontes para este parágrafo |
| `.style(name)` | Aplica um estilo nomeado |
| `.space_before(mm)` / `.space_after(mm)` | Espaçamento explícito em mm |
| `.indent_left(mm)` / `.indent_right(mm)` / `.indent_first_line(mm)` | Indentação |

---

### `Section`

Cabeçalho de secção em três níveis (equivalente a H1–H3).

```rust
use normordis_pdf::Section;

Section::new("Introdução", 1)    // H1 — usa font_size_title + primary_color
Section::new("Subsecção", 2)     // H2 — usa font_size_section
Section::new("Ponto menor", 3)   // H3 — usa font_size_body + negrito
```

Níveis 4–6 do NCRTF são mapeados para nível 3.

---

### `Spacer`

Espaço vertical vazio.

```rust
use normordis_pdf::Spacer;

Spacer::new(5.0)   // 5 mm de espaço vertical
```

---

### `PageBreakElement`

Força uma quebra de página antes do elemento seguinte.

```rust
use normordis_pdf::PageBreakElement;

builder.push(PageBreakElement)
```

---

### `Table`

Tabela com cabeçalho e linhas de dados.

```rust
use normordis_pdf::Table;

let headers = vec!["Nome".into(), "Data".into(), "Valor".into()];
let rows = vec![
    vec!["João Silva".into(), "2026-01-10".into(), "1 200,00 €".into()],
    vec!["Ana Costa".into(), "2026-01-15".into(), "3 450,00 €".into()],
];

Table::new(headers, rows)
    .col_widths(vec![50.0, 25.0, 25.0])  // percentagens da largura útil (devem somar 100)
```

Sem `col_widths`, as colunas têm largura igual.

---

### Listas

**Lista de pontos (`BulletList`)**

```rust
use normordis_pdf::{BulletList, ListItemElement, TextRun};

BulletList {
    items: vec![
        ListItemElement { indent: 0, runs: vec![TextRun::plain("Primeiro ponto")] },
        ListItemElement { indent: 0, runs: vec![TextRun::plain("Segundo ponto")] },
        ListItemElement { indent: 1, runs: vec![TextRun::plain("Sub-ponto")] },
    ],
}
```

**Lista numerada (`OrderedList`)**

```rust
use normordis_pdf::{OrderedList, ListItemElement, TextRun};

OrderedList {
    start: 1,   // número inicial
    items: vec![
        ListItemElement { indent: 0, runs: vec![TextRun::plain("Artigo 1.º")] },
    ],
}
```

**Lista de verificação (`CheckList`)**

```rust
use normordis_pdf::{CheckList, CheckListItem, TextRun};

CheckList {
    items: vec![
        CheckListItem { checked: true,  indent: 0, runs: vec![TextRun::plain("Concluído")] },
        CheckListItem { checked: false, indent: 0, runs: vec![TextRun::plain("Pendente")] },
    ],
}
```

---

### `ImageElement`

Imagem inline no fluxo.

```rust
use normordis_pdf::{ImageElement, ImageAlignment};

let png_bytes: Vec<u8> = std::fs::read("logo.png")?;

ImageElement::new(png_bytes)
    .width(60.0)            // mm (opcional)
    .align(ImageAlignment::Center)
    .caption("Fig. 1 — Logótipo")
```

---

### `InstitutionalHeader`

Cabeçalho institucional — renderizado automaticamente no topo de cada página.

```rust
use normordis_pdf::InstitutionalHeader;

let logo_bytes: Vec<u8> = std::fs::read("logo.png")?;

builder.header(
    InstitutionalHeader::new("Câmara Municipal de Lisboa", "Acta de Reunião")
        .with_subtitle("Reunião Extraordinária")
        .with_logo(logo_bytes)
        .with_reference("REF/2026/042")
        .with_date("24 de Abril de 2026")
)
```

---

### `PageFooter`

Rodapé no fundo de cada página.

```rust
use normordis_pdf::PageFooter;

builder.footer(
    PageFooter::new()
        .left("REF/2026/042")
        .center("Câmara Municipal de Lisboa")
        // show_page_number e show_date também disponíveis
)

// Ou apenas com número de página:
builder.footer(PageFooter::with_page_numbers())
```

---

## 5. Elementos fixos

Elementos fixos são colocados em coordenadas absolutas. **Não afectam o cursor de fluxo.** São renderizados na página corrente no momento em que aparecem na sequência de elementos.

### `FixedBox` — definição da caixa

Todos os elementos fixos recebem um `FixedBox` que define o rectângulo.

```rust
use normordis_pdf::{FixedBox, OverflowPolicy, BoxBorder, BorderStyle, RgbColor};

let box_def = FixedBox {
    x_mm: 20.0,          // distância da margem esquerda da página
    y_mm: 257.0,         // distância da margem inferior da página
    width_mm: 120.0,
    height_mm: 15.0,
    padding_mm: 2.0,     // padding interior (todos os lados)
    overflow: OverflowPolicy::Truncate,
    border: Some(BoxBorder {
        width_mm: 0.3,
        color: RgbColor::from_hex("#CCCCCC").unwrap(),
        style: BorderStyle::Solid,
    }),
    background: RgbColor::from_hex("#F5F5F5"),
};

// Ou com defaults (50×10 mm, padding 2 mm, sem borda nem fundo):
let box_def = FixedBox {
    x_mm: 20.0, y_mm: 100.0,
    width_mm: 80.0, height_mm: 12.0,
    ..Default::default()
};
```

**Métodos de `FixedBox`:**

| Método | Descrição |
|---|---|
| `inner_width_mm()` | Largura útil após padding |
| `inner_height_mm()` | Altura útil após padding |
| `inner_x_mm()` | X do início da área interior |
| `inner_y_top_mm()` | Y do topo da área interior (coordenada de topo do conteúdo) |

### `OverflowPolicy`

| Variante | Comportamento |
|---|---|
| `Truncate` (padrão) | Para de renderizar linhas que ultrapassem o fundo da caixa |
| `Clip` | Aplica clipping path PDF — conteúdo cortado na fronteira |
| `Shrink` | Reduz o tamanho da fonte em passos de 0,5 pt (mínimo 6 pt) até o conteúdo caber |
| `Overflow` | Permite que o conteúdo ultrapasse — útil em desenvolvimento |

---

### `FixedTextBox`

Caixa de texto em posição fixa.

```rust
use normordis_pdf::{FixedTextBox, VerticalAlign, ParagraphContent, TextAlignment, FixedBox};

// Via builder (mais simples):
builder.fixed_text(
    FixedBox { x_mm: 65.0, y_mm: 257.0, width_mm: 125.0, height_mm: 10.0, ..Default::default() },
    "CÂMARA MUNICIPAL DE EXEMPLO",
    TextAlignment::Left,
)

// Ou directamente:
builder.push(FixedTextBox {
    text_box: FixedBox { x_mm: 20.0, y_mm: 22.0, width_mm: 70.0, height_mm: 6.0, ..Default::default() },
    content: ParagraphContent::Plain("Assinatura do Presidente".into()),
    alignment: TextAlignment::Center,
    font_size: Some(9.0),
    vertical_align: VerticalAlign::Middle,
})
```

**`VerticalAlign`:**

| Variante | Comportamento |
|---|---|
| `Top` (padrão) | Conteúdo alinhado ao topo da área interior |
| `Middle` | Centrado verticalmente |
| `Bottom` | Alinhado ao fundo |

**Métodos auxiliares (úteis em testes e geração dinâmica):**

```rust
// Tamanho de fonte efectivo após aplicar Shrink (se aplicável):
let fs = fixed_text_box.effective_font_size(&ctx);

// Y de início do primeiro linha dado o height do conteúdo:
let y = fixed_text_box.content_y_start_mm(content_height_mm);
```

---

### `FixedImageBox`

Imagem em posição fixa.

```rust
use normordis_pdf::{FixedImageBox, ImageFit, FixedBox};

// Via builder:
builder.fixed_image(
    FixedBox { x_mm: 20.0, y_mm: 257.0, width_mm: 40.0, height_mm: 20.0, ..Default::default() },
    logo_bytes,
    ImageFit::Contain,
)
```

**`ImageFit`:**

| Variante | Comportamento |
|---|---|
| `Contain` (padrão) | Escala para caber na caixa, preserva proporção |
| `Cover` | Escala para preencher a caixa, pode cortar |
| `Stretch` | Estica para preencher exactamente (ignora proporção) |
| `Original` | Tamanho original; aplica `OverflowPolicy` se exceder |

---

### `FixedLineElement`

Linha decorativa em posição fixa.

```rust
use normordis_pdf::{FixedLineElement, RgbColor};

// Via builder:
builder.fixed_line(
    20.0, 252.0,   // x1, y1 (mm desde o canto inferior esquerdo)
    190.0, 252.0,  // x2, y2
    RgbColor::from_hex("#003399").unwrap(),
)

// Directamente (permite controlar espessura e estilo):
use normordis_pdf::{BorderStyle};

builder.push({
    let mut line = FixedLineElement::new(20.0, 30.0, 190.0, 30.0,
        RgbColor::new(0.0, 0.0, 0.0));
    line.width_mm = 0.5;
    line.style = BorderStyle::Dashed;
    line
})
```

---

### Exemplo completo — documento misto

```rust
use normordis_pdf::*;

let pdf = DocumentBuilder::new("Certidão de Residência")
    // ── Elementos fixos do template (não perturbam o fluxo) ──
    .fixed_image(
        FixedBox { x_mm: 20.0, y_mm: 257.0, width_mm: 40.0, height_mm: 20.0, ..Default::default() },
        std::fs::read("logo.png")?,
        ImageFit::Contain,
    )
    .fixed_text(
        FixedBox { x_mm: 65.0, y_mm: 261.0, width_mm: 125.0, height_mm: 10.0, ..Default::default() },
        "CÂMARA MUNICIPAL DE EXEMPLO",
        TextAlignment::Left,
    )
    .fixed_line(20.0, 252.0, 190.0, 252.0, RgbColor::from_hex("#003399").unwrap())
    .fixed_text(
        FixedBox { x_mm: 155.0, y_mm: 15.0, width_mm: 35.0, height_mm: 8.0, ..Default::default() },
        "Ref: 2026/001",
        TextAlignment::Left,
    )
    // ── Conteúdo em fluxo ──
    .push(Spacer::new(50.0))   // reserva espaço para o cabeçalho fixo
    .push(Section::new("Certidão de Residência", 1))
    .push(Paragraph::new("Certifica-se que o cidadão...").align(TextAlignment::Justify))
    .push(Spacer::new(20.0))
    .push(Paragraph::new("Local e data: Lisboa, 24 de Abril de 2026"))
    // ── Zona de assinatura fixa ──
    .fixed_line(20.0, 30.0, 90.0, 30.0, RgbColor::from_hex("#000000").unwrap())
    .fixed_text(
        FixedBox { x_mm: 20.0, y_mm: 22.0, width_mm: 70.0, height_mm: 6.0, ..Default::default() },
        "Assinatura do Presidente",
        TextAlignment::Center,
    )
    .render_to_bytes()?;
```

---

## 6. Formato NCRTF

NCRTF (*Normaxis Canonical Rich Text Format*) é o formato JSON interno para rich text editável. O `DocumentBuilder::push_ncrtf()` parseia o JSON e converte automaticamente para elementos.

### Estrutura de topo

```json
{
  "ncrtf": "1.0",
  "meta": { "title": "Exemplo", "lang": "pt" },
  "blocks": [ ...Block[] ]
}
```

### Tipos de bloco (`Block`)

| `"type"` | Mapeamento | Campos principais |
|---|---|---|
| `"paragraph"` | `Paragraph` | `alignment`, `children` |
| `"heading"` | `Section` | `level` (1–6), `children` |
| `"list"` | `BulletList` / `OrderedList` / `CheckList` | `list_type`, `children` |
| `"table"` | `Table` | `head`, `body`, `col_widths` |
| `"blockquote"` | `Paragraph` com indent 8 mm | `children` |
| `"code_block"` | `Paragraph` monoespaçado | `code`, `language` |
| `"image"` | `ImageElement` | `src`, `caption`, `alignment` |
| `"horizontal_rule"` | `Spacer` de 2 mm | — |
| `"page_break"` | `PageBreakElement` | — |
| `"fixed_box"` | `FixedTextBox` | `x_mm`, `y_mm`, `width_mm`, `height_mm`, `children`, `alignment`, `overflow`, `padding_mm`, `border`, `background` |

### Tipos de inline (`Inline`)

| `"type"` | Descrição |
|---|---|
| `"text"` | Texto com marks formatados |
| `"link"` | Texto com hiperligação (filhos são inlines) |
| `"hard_break"` | Quebra de linha forçada |

### Marks de texto

```json
{ "type": "text", "text": "Exemplo ", "marks": ["bold", "italic"] }
{ "type": "text", "text": "Azul",     "marks": [{"type": "color", "value": "#0033CC"}] }
{ "type": "text", "text": "Grande",   "marks": [{"type": "font_size", "value": 14.0}] }
```

Marks simples (string): `"bold"`, `"italic"`, `"underline"`, `"strikethrough"`, `"superscript"`, `"subscript"`, `"code"`.

Marks com parâmetro (objecto `{type, value}`): `"color"`, `"highlight"`, `"font_size"`.

### Exemplo completo NCRTF

```json
{
  "ncrtf": "1.0",
  "meta": { "title": "Acta n.º 5/2026" },
  "blocks": [
    {
      "type": "heading",
      "level": 1,
      "alignment": "left",
      "children": [{ "type": "text", "text": "Acta da Reunião", "marks": [] }]
    },
    {
      "type": "paragraph",
      "alignment": "justify",
      "children": [
        { "type": "text", "text": "Aos ", "marks": [] },
        { "type": "text", "text": "24 de Abril de 2026", "marks": ["bold"] },
        { "type": "text", "text": ", reuniu a Assembleia...", "marks": [] }
      ]
    },
    { "type": "page_break" },
    {
      "type": "fixed_box",
      "x_mm": 20.0, "y_mm": 30.0,
      "width_mm": 80.0, "height_mm": 8.0,
      "alignment": "center",
      "children": [{ "type": "text", "text": "Presidente da Câmara", "marks": [] }]
    }
  ]
}
```

### Parsear e converter manualmente

```rust
use normordis_pdf::{parse_ncrtf, ncrtf_to_elements, DocumentStyle};

let doc = parse_ncrtf(json_str)?;     // -> NcrtfDocument
let elements = ncrtf_to_elements(&doc, &DocumentStyle::default());
// elements: Vec<Box<dyn Element>>
```

---

## 7. Fontes

### Fontes embebidas

Quatro famílias estão embebidas em tempo de compilação via `include_bytes!` — nenhuma dependência de sistema é necessária:

| Nome | Equivalente Word | Uso típico |
|---|---|---|
| `LiberationSans` | Arial / Calibri / Helvetica | Corpo padrão (default) |
| `LiberationSerif` | Times New Roman / Cambria / Georgia | Texto formal com serifa |
| `LiberationMono` | Courier New / Consolas | Código, referências |
| `LibertinusSerif` | — | Corpo com serifa alternativa |

Aliases Word pré-configurados: `Arial`, `Calibri`, `Helvetica` → `LiberationSans`; `Times New Roman`, `Cambria`, `Georgia` → `LiberationSerif`; `Courier New`, `Consolas` → `LiberationMono`.

### Registar fontes via `DocumentBuilder`

O método mais simples — regista diretamente no builder sem criar um `FontRegistry` manual:

```rust
use normordis_pdf::{DocumentBuilder, Paragraph};

// A partir de bytes em memória (e.g. include_bytes!)
let pdf = DocumentBuilder::new("Documento")
    .font_from_bytes(
        "GilSans",
        include_bytes!("assets/GilSans-Regular.ttf"),
        Some(include_bytes!("assets/GilSans-Bold.ttf")),
        Some(include_bytes!("assets/GilSans-Italic.ttf")),
        None, // sem Bold Italic
    )?
    .push(Paragraph::new("Texto em GilSans.").font_family("GilSans"))
    .render_to_bytes()?;

// A partir de ficheiros em disco
let pdf = DocumentBuilder::new("Documento")
    .font_from_file(
        "FiraCode",
        "assets/FiraCode-Regular.ttf",
        None::<&str>, None::<&str>, None::<&str>,
    )?
    .push(Paragraph::new("Código em FiraCode.").font_family("FiraCode"))
    .render_to_bytes()?;

// Carregar todas as fontes de um directório
let pdf = DocumentBuilder::new("Documento")
    .fonts_from_dir("assets/fonts/")?
    .render_to_bytes()?;

// Alterar a fonte por omissão
let pdf = DocumentBuilder::new("Documento")
    .default_font("LiberationSerif")?
    .push(Paragraph::new("Serif por omissão."))
    .render_to_bytes()?;
```

### Registar fontes via `FontRegistry`

Para cenários mais avançados (registar fontes antes de criar o builder, partilhar um registo entre documentos):

```rust
use normordis_pdf::{FontRegistry, DocumentBuilder, Paragraph};

let mut reg = FontRegistry::default(); // inclui as quatro famílias Liberation

// register_bytes — aceita &[u8]
reg.register_bytes(
    "Crimson",
    include_bytes!("assets/Crimson-Regular.ttf"),
    Some(include_bytes!("assets/Crimson-Bold.ttf")),
    None, None,
)?;

// register_file — aceita caminhos no sistema de ficheiros
reg.register_file(
    "Montserrat",
    "assets/Montserrat-Regular.ttf",
    Some("assets/Montserrat-Bold.ttf"),
    None::<&str>, None::<&str>,
)?;

// register_single — variante única sem bold/italic (fontes de ícones, display, etc.)
reg.register_single("Icons", "assets/icons.ttf")?;

// Carregar um directório inteiro (agrupa por sufixo: -Regular, -Bold, -Italic, -BoldItalic)
let count = reg.load_dir("assets/fonts/")?;
println!("Carregadas {count} famílias");

// Aliases personalizados
reg.add_alias("Helvetica Neue", "Montserrat");

let pdf = DocumentBuilder::new("Documento")
    .fonts(reg)
    .push(Paragraph::new("Serif.").font_family("Crimson"))
    .render_to_bytes()?;
```

### `FontRegistry` — referência rápida

| Método | Descrição |
|---|---|
| `FontRegistry::default()` | Cria com as quatro famílias Liberation embebidas |
| `FontRegistry::empty()` | Cria sem nenhuma fonte registada |
| `register_bytes(name, regular, bold?, italic?, bold_italic?)` | A partir de `&[u8]` |
| `register_file(name, regular, bold?, italic?, bold_italic?)` | A partir de caminhos TTF/OTF |
| `register_single(name, path)` / `register_single_bytes(name, bytes)` | Variante única |
| `register(FontVariants)` | Regista diretamente um `FontVariants` já construído |
| `load_dir(dir)` | Adiciona famílias de um directório; devolve `usize` (nº de famílias) |
| `add_alias(alias, target)` | Ex.: `"Helvetica"` → `"LiberationSans"` |
| `set_default(name)` | Define a família usada quando não há `font_family` explícito |
| `get_family(name)` | Resolve nome (incluindo aliases); nunca devolve `None` |
| `try_resolve(name, depth)` | Resolução com proteção de ciclos (máx. 8 saltos); `Option` |
| `resolve(name)` | Como `try_resolve` mas nunca `None` |
| `contains(name)` | `true` se o nome está registado direta ou por alias |
| `registered_families()` | `Vec<&str>` com todos os nomes directamente registados |

### Fonte por parágrafo

Qualquer parágrafo pode substituir a fonte do estilo nomeado ou a fonte por omissão:

```rust
use normordis_pdf::Paragraph;

Paragraph::new("Texto serifado.").font_family("LiberationSerif")
Paragraph::new("Monospace.").font_family("LiberationMono")
Paragraph::new("Fonte customizada.").font_family("GilSans")
// Se "GilSans" não estiver registada, usa a cadeia de fallback e emite warning
```

O override de `font_family` também funciona com NDT através do campo `font_family` nos estilos nomeados:

```toml
# template.ndt.toml
[styles.corpo_serifado]
extends     = "normal"
font_family = "LiberationSerif"
font_size   = 11.0
```

### Cadeia de fallback (`FontFallbackChain`)

Quando a fonte pedida não está registada, o engine percorre a cadeia de fallback em sequência e usa a primeira que encontrar. Se nenhuma estiver disponível, usa a fonte por omissão do documento. Um `eprintln!` de aviso é emitido — nunca é devolvido um erro.

```rust
use normordis_pdf::{DocumentStyle, FontFallbackChain};

let mut style = DocumentStyle::default();
// Por omissão: ["LiberationSans", "LiberationSerif", "LiberationMono"]
// Personalizar:
style.font_fallback = FontFallbackChain::new(vec![
    "Crimson",        // serif registada
    "LiberationSans", // fallback final
]);
```

A cadeia é serializável/desserializável (Serde) e pode ser definida num template NDT:

```json
{
  "font_fallback": { "fonts": ["LiberationSerif", "LiberationSans"] }
}
```

### Fontes do sistema (`feature = "system-fonts"`)

Com a feature `system-fonts` activa, é possível descobrir fontes instaladas no sistema operativo. **Não recomendado para produção** (resultados dependem do SO):

```toml
# Cargo.toml
normordis-pdf = { version = "...", features = ["system-fonts"] }
```

```rust
#[cfg(feature = "system-fonts")]
{
    let reg = FontRegistry::from_system()?;
    // "Calibri", "Segoe UI", etc. ficam disponíveis se instaladas
}
```

---

## 8. Tratamento de erros

Todos os métodos que podem falhar devolvem `normordis_pdf::Result<T>`, alias para `Result<T, NormaxisPdfError>`.

```rust
use normordis_pdf::NormaxisPdfError;

match builder.render_to_bytes() {
    Ok(bytes) => { /* ... */ }
    Err(NormaxisPdfError::ParseError(msg)) => eprintln!("JSON inválido: {msg}"),
    Err(NormaxisPdfError::RenderError(msg)) => eprintln!("Falha de renderização: {msg}"),
    Err(NormaxisPdfError::IoError(e)) => eprintln!("Erro de I/O: {e}"),
    Err(e) => eprintln!("Erro: {e}"),
}
```

**Variantes de `NormaxisPdfError`:**

| Variante | Causa |
|---|---|
| `ParseError(String)` | JSON NCRTF inválido |
| `RenderError(String)` | Falha interna de renderização (e.g. PDF vazio) |
| `FontLoadError(String)` | Erro ao carregar bytes de fonte |
| `ImageLoadError(String)` | Imagem inválida ou formato não suportado |
| `IoError(std::io::Error)` | Erro de leitura/escrita de ficheiro |

---

## 9. Implementar um elemento personalizado

Qualquer struct que implemente `Element` pode ser adicionada via `builder.push()`.

```rust
use normordis_pdf::{Element, LayoutMode, RenderContext};

struct MeuElemento {
    altura_mm: f64,
}

impl Element for MeuElemento {
    // layout_mode() tem default LayoutMode::Flow — não é necessário substituir
    // para elementos de fluxo normais.

    fn estimated_height_mm(&self) -> f64 {
        self.altura_mm
    }

    fn render(&self, ctx: &mut RenderContext) -> normordis_pdf::Result<()> {
        // Empurrar ops printpdf para ctx.ops, por exemplo:
        // ctx.ops.push(printpdf::Op::SetFillColor { col: ... });
        // ctx.ops.push(printpdf::Op::Rect { ... });
        // ctx.ops.push(printpdf::Op::FillPath);

        // Avançar o cursor (obrigatório para elementos de fluxo):
        ctx.flow.advance(self.altura_mm);

        Ok(())
    }
}
```

Para um elemento fixo, substituir `layout_mode()`:

```rust
fn layout_mode(&self) -> LayoutMode {
    LayoutMode::Fixed(FixedBox {
        x_mm: 10.0, y_mm: 50.0,
        width_mm: 80.0, height_mm: 20.0,
        ..Default::default()
    })
}
```

Num elemento fixo, **não chamar** `ctx.flow.advance()`.

---

## 10. Sistema de coordenadas

| Conceito | Referência | Sentido |
|---|---|---|
| Origem | Canto inferior esquerdo da página | — |
| `x_mm` | Distância do lado esquerdo | →  esquerda para a direita |
| `y_mm` | Distância do fundo | ↑ de baixo para cima |
| `cursor_y_mm` | Y corrente do fluxo | começa perto do topo, **decresce** com conteúdo |
| Margens | Excluídas da área útil | `content_x = margin_left`, `top_y = height - margin_top` |

**Conversões úteis:**

```
y_a_partir_do_topo = page_height - margin_top - deslocamento_desde_o_topo
y_a_partir_do_fundo = margin_bottom + deslocamento_desde_o_fundo
```

Para elementos fixos em rodapé (zona de assinatura, referências):
- Zona de assinatura a 30 mm do fundo → `y_mm = 30.0`
- Rodapé a 15 mm do fundo → `y_mm = 15.0`

Para elementos fixos em cabeçalho (A4, 297 mm):
- Linha separadora a 252 mm do fundo → `y_mm = 252.0` (a 45 mm do topo)
- Logo a 257 mm do fundo → `y_mm = 257.0` (a 40 mm do topo)

