# Registo de decisões de arquitetura

Formato e regras em `README.md` neste diretório. Entradas por ordem
cronológica de decisão. As três primeiras foram registadas em 2026-09-04,
data de criação deste registo, a partir de decisões já tomadas e visíveis
no histórico; as seguintes entram no momento em que são tomadas.

---

## ADR-001 — Recusar requisitos mutuamente exclusivos em vez de emitir ficheiro não conforme

- **Data:** 2026-09-03
- **Estado:** aceite
- **Decisão de:** Carlos Canuto Costa
- **Proposta por:** responsável
- **Origem:** commit `c813baf`; ISO 19005-1 §6.4

### Contexto
PDF/A-1b proíbe transparência. A marca de água translúcida usada nos
documentos institucionais exige transparência. A biblioteca podia emitir o
ficheiro em silêncio com a declaração PDF/A-1b, ou recusar.

### Decisão
Quando dois requisitos são mutuamente exclusivos, a biblioteca **recusa** e
explica, citando a cláusula da norma. Nunca emite um ficheiro que declara
uma conformidade que não tem. A marca translúcida passa a exigir PDF/A-2b.

### Alternativas rejeitadas
Emitir com aviso no log: a conformidade declarada no ficheiro é o que uma
instituição verifica, não o log. Degradar a marca para opaca: altera o
documento sem o autor saber.

### Consequências
Regra geral de desenho, aplicável a qualquer futuro conflito norma versus
funcionalidade. O validador externo (veraPDF) é a referência final.

---

## ADR-002 — Renomear o formato de arquivo interno para não colidir com o NDF normativo

- **Data:** 2026-09-04
- **Estado:** aceite
- **Decisão de:** Carlos Canuto Costa
- **Proposta por:** responsável, a partir do achado R14
- **Origem:** achado R14 de `docs/reports/READINESS-ASSESSMENT.md` em `normordis-formats`; CHANGELOG 3.0.0

### Contexto
A crate chamava "NDF" ao seu formato de arquivo de renderização
(`ndf: "1.1.0"`). O NDF-core 1.0.0 especificado em `normordis-formats` é
outro formato, sem um único campo em comum. Dois formatos com o mesmo nome
no mesmo ecossistema é um erro de nomenclatura com custo real para quem
audita.

### Decisão
O formato interno passa a chamar-se `RenderArchive` (módulo `archive`,
prefixo `Archive*`, `ARCHIVE_VERSION`). O nome NDF fica reservado ao
formato da especificação. Ficheiros antigos continuam legíveis por alias de
campo. Detalhe da renomeação no CHANGELOG 3.0.0.

### Alternativas rejeitadas
Manter o nome e documentar a diferença: a confusão persistiria em cada
integração. Alinhar o arquivo interno ao NDF-core: são artefactos de
natureza diferente (arquivo de renderização versus instância documental).

### Consequências
Versão 3.0.0 com quebra de API pública. Compatibilidade de leitura mantida.

---

## ADR-003 — normordis-formats como referência exclusiva a partir da 4.0.0

- **Data:** 2026-09-04
- **Estado:** aceite
- **Decisão de:** Carlos Canuto Costa
- **Proposta por:** responsável
- **Origem:** CHANGELOG 3.0.0; README, secção "Formatos NORMORDIS"

### Contexto
Até à 3.0.0 a crate definia por si própria as estruturas NDF, NDT e NCRTF,
em paralelo com as especificações publicadas em `normordis-formats`. Duas
fontes de verdade para o mesmo formato divergem com o tempo.

### Decisão
A 3.0.0 é a última versão em que a crate define os formatos. A partir da
4.0.0, as especificações e schemas publicados em `normordis-formats` são a
referência exclusiva, e a crate valida contra eles. Esta crate passa a ser
apenas a representação visual (PDF/A, PDF/UA) do que o NDF define, em linha
com o princípio "NDF é a fonte de verdade documental".

### Consequências
O plano de migração e as decisões que dele resultarem (validação de schema,
tratamento de versões, renderizador NDT) entram como ADR próprios neste
registo à medida que forem tomados.

---

## ADR-004 — Política de utilização de IA generativa e registo de proveniência

- **Data:** 2026-09-04
- **Estado:** aceite
- **Decisão de:** Carlos Canuto Costa
- **Proposta por:** agente (Claude Fable 5.1, Claude Code), sobre rascunhos anteriores do responsável
- **Origem:** política de IA generativa da NLnet v1.1 (2026-01-26); preparação de candidatura ao NGI Zero

### Contexto
O projeto é desenvolvido com assistência de IA generativa. A política da
NLnet exige transparência sobre esse uso, registo de proveniência e
contribuição humana substancial em cada entregável. Instituições públicas
podem confundir "desenvolvido com IA" com "contém IA".

### Decisão
Adotar `AI_USAGE.md` (política), `docs/ai-provenance.md` (convenção de
commits com nome e versão exatos do modelo) e
`docs/genai-application-disclosure-template.md` (registo de prompts da
candidatura), em vigor desde 2026-09-04, sem retroatividade. Declarar
explicitamente que o produto não é um sistema de IA nem incorpora modelos
em runtime, pelo que o Regulamento (UE) 2024/1689 não se lhe aplica.
Tornar a contribuição humana estruturalmente visível através da secção 12
do `AGENTS.md` e deste registo de decisões.

### Alternativas rejeitadas
Não declarar o uso de IA: viola a política da NLnet e cria risco de
credibilidade perante qualquer avaliador. Registar retroativamente todo o
histórico: a política não o exige e o resultado seria reconstrução, não
registo.

### Consequências
Cada commit assistido passa a exigir revisão humana registada. Testes de
conformidade não são escritos na sessão que implementou o código testado.
Este registo de decisões passa a ser obrigatório para decisões de
arquitetura, formato, norma e dependência.
