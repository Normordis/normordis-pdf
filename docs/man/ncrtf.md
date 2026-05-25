# NCRTF — Normordis Craft Rich Text Format

**Versão:** 1.3.0  
**MIME:** `application/vnd.normordis.ncrtf+json`  
**Encoding:** JSON UTF-8  
**Autoridade:** `normordis-pdf` (este repositório)

---

## 1. Propósito

NCRTF é o formato de rich text proprietário da plataforma Normordis. É o contrato
entre o editor e qualquer renderizador, exportador ou sistema de custódia documental.

Qualquer editor (Lexical, Quill, etc.) produz NCRTF através do seu serializador
específico. O `normordis-pdf` consome NCRTF directamente para renderizar PDF.
Outros formatos derivados (NDF, NDT) são construídos a partir de NCRTF pelo
domínio documental.

```
editor rich text  →  NCRTF  →  normordis-pdf  →  PDF
                          ↓
                       NDF / NDT  (domínio documental)
```

---

## 2. Estrutura raiz

```json
{
  "ncrtf": "1.3.0",
  "meta": { ... },
  "blocks": [ ... ]
}
```

| Campo    | Tipo     | Obrigatório | Descrição                        |
|----------|----------|-------------|----------------------------------|
| `ncrtf`  | `string` | Sim         | Versão do formato, ex. `"1.3.0"` |
| `meta`   | `object` | Não         | Metadados do documento           |
| `blocks` | `array`  | Sim         | Sequência de blocos de conteúdo  |

### 2.1 Meta

```json
{
  "title": "Contrato de Prestação de Serviços",
  "lang": "pt",
  "author": "Carlos Costa",
  "created_at": "2026-05-25T10:00:00Z",
  "updated_at": "2026-05-25T14:30:00Z",
  "custom": { "reference": "2026/001", "department": "Jurídico" }
}
```

Todos os campos de `meta` são opcionais. O campo `custom` aceita qualquer
objecto JSON para metadata arbitrária da aplicação consumidora.

---

## 3. Blocos

Cada bloco tem um campo `"type"` obrigatório que determina a sua estrutura.

### 3.1 `paragraph`

Parágrafo de texto com inlines.

```json
{
  "type": "paragraph",
  "alignment": "justify",
  "indent": 1,
  "style": "body",
  "children": [ ... ]
}
```

| Campo       | Tipo       | Obrigatório | Valores                                    |
|-------------|------------|-------------|-------------------------------------------|
| `alignment` | `string`   | Não         | `"left"` `"center"` `"right"` `"justify"` |
| `indent`    | `integer`  | Não         | 0–10                                       |
| `style`     | `string`   | Não         | Identificador de estilo tipográfico        |
| `children`  | `Inline[]` | Sim         | Nodos inline                               |

### 3.2 `heading`

Título com nível hierárquico.

```json
{
  "type": "heading",
  "level": 1,
  "alignment": "left",
  "children": [ ... ]
}
```

| Campo       | Tipo       | Obrigatório | Valores                                    |
|-------------|------------|-------------|-------------------------------------------|
| `level`     | `integer`  | Sim         | 1–6                                        |
| `alignment` | `string`   | Não         | `"left"` `"center"` `"right"` `"justify"` |
| `children`  | `Inline[]` | Sim         | Nodos inline                               |

### 3.3 `list`

Lista ordenada, não ordenada ou checklist.

```json
{
  "type": "list",
  "list_type": "bullet",
  "children": [
    {
      "indent": 0,
      "checked": null,
      "children": [ { "type": "text", "text": "Item um" } ]
    }
  ]
}
```

| Campo       | Tipo          | Obrigatório | Valores                              |
|-------------|---------------|-------------|--------------------------------------|
| `list_type` | `string`      | Sim         | `"bullet"` `"ordered"` `"checklist"` |
| `children`  | `ListItem[]`  | Sim         | Items da lista                       |

**ListItem:**

| Campo      | Tipo       | Obrigatório | Descrição                                        |
|------------|------------|-------------|--------------------------------------------------|
| `indent`   | `integer`  | Não         | Nível de indentação (para listas aninhadas)      |
| `checked`  | `boolean`  | Não         | `true/false` para checklist; `null` ou ausente noutros casos |
| `children` | `Inline[]` | Sim         | Conteúdo inline do item                          |

### 3.4 `table`

Tabela com cabeçalho opcional e corpo.

```json
{
  "type": "table",
  "caption": "Tabela de honorários",
  "col_widths": [40.0, 30.0, 30.0],
  "head": [
    {
      "cells": [
        { "header": true, "children": [ { "type": "text", "text": "Serviço" } ] },
        { "header": true, "children": [ { "type": "text", "text": "Horas" } ] },
        { "header": true, "children": [ { "type": "text", "text": "Valor" } ] }
      ]
    }
  ],
  "body": [
    {
      "cells": [
        { "children": [ { "type": "text", "text": "Consultoria" } ] },
        { "children": [ { "type": "text", "text": "10" } ] },
        { "children": [ { "type": "text", "text": "1 500,00 €" } ] }
      ]
    }
  ]
}
```

| Campo        | Tipo          | Obrigatório | Descrição                                        |
|--------------|---------------|-------------|--------------------------------------------------|
| `caption`    | `string`      | Não         | Legenda da tabela                                |
| `col_widths` | `number[]`    | Não         | Larguras relativas das colunas (percentagem)     |
| `head`       | `TableRow[]`  | Sim         | Linhas de cabeçalho (pode ser `[]`)              |
| `body`       | `TableRow[]`  | Sim         | Linhas de dados                                  |

**TableRow:** `{ "cells": TableCell[] }`

**TableCell:**

| Campo       | Tipo       | Obrigatório | Descrição                                    |
|-------------|------------|-------------|----------------------------------------------|
| `header`    | `boolean`  | Não         | `true` se a célula é de cabeçalho            |
| `col_span`  | `integer`  | Não         | Expansão em colunas                          |
| `row_span`  | `integer`  | Não         | Expansão em linhas                           |
| `alignment` | `string`   | Não         | Alinhamento do conteúdo da célula            |
| `children`  | `Inline[]` | Sim         | Conteúdo inline da célula                    |

### 3.5 `blockquote`

Citação em bloco.

```json
{
  "type": "blockquote",
  "attribution": "Autor, Obra, 1984",
  "children": [ ... ]
}
```

| Campo         | Tipo       | Obrigatório | Descrição          |
|---------------|------------|-------------|--------------------|
| `attribution` | `string`   | Não         | Fonte da citação   |
| `children`    | `Inline[]` | Sim         | Conteúdo inline    |

### 3.6 `code_block`

Bloco de código literal.

```json
{
  "type": "code_block",
  "language": "json",
  "code": "{ \"ncrtf\": \"1.3.0\" }"
}
```

| Campo      | Tipo     | Obrigatório | Descrição                         |
|------------|----------|-------------|-----------------------------------|
| `language` | `string` | Não         | Identificador da linguagem        |
| `code`     | `string` | Sim         | Conteúdo literal (sem escape)     |

### 3.7 `image`

Imagem embutida ou por referência.

```json
{
  "type": "image",
  "src": "data:image/png;base64,...",
  "alt": "Logótipo Normordis",
  "caption": "Figura 1 — Logótipo institucional",
  "alignment": "center",
  "width_percent": 50.0
}
```

| Campo           | Tipo     | Obrigatório | Descrição                                        |
|-----------------|----------|-------------|--------------------------------------------------|
| `src`           | `string` | Sim         | Data URI (`data:...`) ou referência (`asset:...`). URLs HTTP/HTTPS **não são aceites** |
| `alt`           | `string` | Não         | Texto alternativo                                |
| `caption`       | `string` | Não         | Legenda visível                                  |
| `alignment`     | `string` | Não         | `"left"` `"center"` `"right"`                   |
| `width_percent` | `number` | Não         | Largura em percentagem da área disponível        |

### 3.8 `horizontal_rule`

Linha horizontal separadora. Não tem campos adicionais.

```json
{ "type": "horizontal_rule" }
```

### 3.9 `page_break`

Quebra de página explícita. Não tem campos adicionais.

```json
{ "type": "page_break" }
```

### 3.10 `fixed_box`

Caixa de texto posicionada de forma absoluta na página (em mm).

```json
{
  "type": "fixed_box",
  "x_mm": 20.0,
  "y_mm": 240.0,
  "width_mm": 80.0,
  "height_mm": 20.0,
  "overflow": "truncate",
  "padding_mm": 2.0,
  "border": { "width_mm": 0.25, "color": "#CCCCCC", "style": "solid" },
  "background": "#F5F5F5",
  "alignment": "center",
  "children": [ ... ]
}
```

| Campo        | Tipo       | Obrigatório | Descrição                                              |
|--------------|------------|-------------|--------------------------------------------------------|
| `x_mm`       | `number`   | Sim         | Posição horizontal desde a margem esquerda             |
| `y_mm`       | `number`   | Sim         | Posição vertical desde o topo da página                |
| `width_mm`   | `number`   | Sim         | Largura da caixa                                       |
| `height_mm`  | `number`   | Sim         | Altura da caixa                                        |
| `overflow`   | `string`   | Não         | `"truncate"` `"clip"` `"shrink"` `"overflow"`         |
| `padding_mm` | `number`   | Não         | Espaço interior uniforme                               |
| `border`     | `object`   | Não         | `{ width_mm, color, style }` — style: `"solid"` `"dashed"` `"dotted"` |
| `background` | `string`   | Não         | Cor de fundo em hex, ex. `"#F5F5F5"`                  |
| `alignment`  | `string`   | Não         | Alinhamento do texto interno                           |
| `children`   | `Inline[]` | Sim         | Conteúdo inline                                        |

---

## 4. Inlines

### 4.1 `text`

Fragmento de texto com marcas de formatação.

```json
{
  "type": "text",
  "text": "Texto em negrito e itálico",
  "marks": ["bold", "italic"]
}
```

| Campo    | Tipo      | Obrigatório | Descrição              |
|----------|-----------|-------------|------------------------|
| `text`   | `string`  | Sim         | Conteúdo textual       |
| `marks`  | `Mark[]`  | Não         | Formatações aplicadas  |

### 4.2 `link`

Hiperligação inline com conteúdo rico.

```json
{
  "type": "link",
  "href": "https://normordis.pt",
  "title": "Normordis",
  "target": "_blank",
  "children": [ { "type": "text", "text": "normordis.pt" } ]
}
```

| Campo      | Tipo       | Obrigatório | Descrição                      |
|------------|------------|-------------|--------------------------------|
| `href`     | `string`   | Sim         | URL de destino                 |
| `title`    | `string`   | Não         | Tooltip                        |
| `target`   | `string`   | Não         | Contexto de abertura           |
| `children` | `Inline[]` | Sim         | Conteúdo visível da ligação    |

### 4.3 `hard_break`

Quebra de linha dentro do parágrafo.

```json
{ "type": "hard_break" }
```

### 4.4 `footnote_ref`

Marcador de referência a nota de rodapé (NCRTF 1.3.0+). Renderizado como
número sobrescrito.

```json
{ "type": "footnote_ref", "number": 1 }
```

---

## 5. Marcas de texto

As marcas podem ser simples strings ou objectos parametrizados.

### 5.1 Marcas simples (string)

| Marca           | Efeito                         |
|-----------------|--------------------------------|
| `"bold"`        | Negrito                        |
| `"italic"`      | Itálico                        |
| `"underline"`   | Sublinhado                     |
| `"strikethrough"` | Rasurado                     |
| `"superscript"` | Sobrescrito                    |
| `"subscript"`   | Subscrito                      |
| `"code"`        | Código inline (fonte monospace)|
| `"small_caps"`  | Versaletes                     |

### 5.2 Marcas parametrizadas (objecto)

**Cor do texto:**
```json
{ "type": "color", "value": "#CC0000" }
```

**Realce (highlight):**
```json
{ "type": "highlight", "value": "#FFFF00" }
```

**Tamanho de fonte:**
```json
{ "type": "font_size", "value": 14.0 }
```

**Sublinhado com cor:**
```json
{ "type": "underline", "color": "#0000CC" }
```

**Rasurado com cor:**
```json
{ "type": "strikethrough", "color": "#CC0000" }
```

---

## 6. Exemplo completo

```json
{
  "ncrtf": "1.3.0",
  "meta": {
    "title": "Declaração de Responsabilidade",
    "lang": "pt",
    "author": "Serviço Jurídico",
    "updated_at": "2026-05-25T14:00:00Z"
  },
  "blocks": [
    {
      "type": "heading",
      "level": 1,
      "children": [
        { "type": "text", "text": "Declaração de Responsabilidade" }
      ]
    },
    {
      "type": "paragraph",
      "alignment": "justify",
      "children": [
        { "type": "text", "text": "O signatário, " },
        { "type": "text", "text": "João Silva", "marks": ["bold"] },
        { "type": "text", "text": ", declara para todos os efeitos legais que:" }
      ]
    },
    {
      "type": "list",
      "list_type": "ordered",
      "children": [
        {
          "children": [
            { "type": "text", "text": "Os dados fornecidos são verídicos;" }
          ]
        },
        {
          "children": [
            { "type": "text", "text": "Assume integral responsabilidade pelo conteúdo." }
          ]
        }
      ]
    },
    {
      "type": "table",
      "head": [
        {
          "cells": [
            { "header": true, "children": [ { "type": "text", "text": "Campo" } ] },
            { "header": true, "children": [ { "type": "text", "text": "Valor" } ] }
          ]
        }
      ],
      "body": [
        {
          "cells": [
            { "children": [ { "type": "text", "text": "NIF" } ] },
            { "children": [ { "type": "text", "text": "123 456 789" } ] }
          ]
        }
      ]
    },
    {
      "type": "paragraph",
      "alignment": "right",
      "children": [
        { "type": "text", "text": "Lisboa, 25 de Maio de 2026" }
      ]
    }
  ]
}
```

---

## 7. Regras de validação

1. O campo `"ncrtf"` deve estar presente e conter uma string de versão válida.
2. `image.src` deve ser uma data URI (`data:...`) ou referência de asset
   (`asset:...`). URLs HTTP/HTTPS são **rejeitadas** pelo renderizador.
3. `heading.level` deve estar no intervalo \[1, 6\].
4. `list_item.checked` só é significativo quando `list.list_type` é `"checklist"`.
5. Blocos desconhecidos devem ser ignorados silenciosamente por versões futuras
   do renderizador (forward compatibility).
6. Inlines desconhecidos devem ser ignorados silenciosamente.

---

## 8. Relação com outros formatos Normordis

| Formato | Papel                                                           | Autoridade        |
|---------|-----------------------------------------------------------------|-------------------|
| NCRTF   | Rich text editável e renderizável                               | `normordis-pdf`   |
| NDF     | Registo imutável em DB com integridade SHA-256 e cadeia de auditoria | `normordis-pdf` |
| NDT     | Template com placeholders resolvíveis                           | `normordis-pdf`   |

Todos os formatos são definidos neste repositório. Outros projectos serializam
**para** estes formatos; não os redefinem.

---

## 9. Histórico de versões

| Versão | Alterações                                                        |
|--------|-------------------------------------------------------------------|
| 1.3.0  | `footnote_ref` inline; `fixed_box` block; `small_caps` mark      |
| 1.2.0  | `col_span`, `row_span` em `TableCell`; `col_widths` em `TableBlock` |
| 1.1.0  | `code_block`; `attribution` em `blockquote`; marcas parametrizadas |
| 1.0.0  | Versão inicial                                                    |
