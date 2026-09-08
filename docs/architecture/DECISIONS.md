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

---

## ADR-005 — Spike krilla como possível motor de conformidade PDF/A + PDF/UA

- **Data:** 2026-09-04
- **Estado:** proposta
- **Decisão de:** Carlos Canuto Costa
- **Proposta por:** agente (Claude Sonnet 5, Claude Code), a partir de verificação direta do código-fonte do krilla em sessão anterior
- **Origem:** pedido do responsável ("Vamos decidir sobre o spike krilla, no próprio repo e registar o ADR"); ADR-003 previa ADR próprio para decisões de dependência resultantes da migração de formatos

### Contexto
`normordis-pdf` produz PDF/A e PDF/UA-2 com motor próprio
(pdf-writer + subsetter + rustybuzz). `krilla` 0.8.2 é uma crate Rust de
alto nível para geração de PDF que reivindica cobertura PDF/A completa
(A1–A4, incl. A4F/A4E) e *tagging*. Verificação direta do código-fonte
(`crates/krilla/src/configure/validate.rs`) mostra que o krilla cobre
apenas **PDF/UA-1** — zero ocorrências de `UA2`/`14289-2` — enquanto
PDF/UA-2 (ISO 14289-2:2024) é o diferenciador central do
`normordis-pdf` e a alegação que a CI valida por veraPDF.

O responsável fixou o critério orientador: o produto tem de cumprir os
requisitos legais da AP e acompanhar a dinâmica legislativa, devendo
ficar o mais independente possível de terceiros no caminho crítico de
conformidade. Isto inverte o ónus da prova: o krilla só é adotado se o
spike demonstrar ganho que compense a dependência — não por omissão.

### Decisão
Correr um spike em branch própria no repositório `normordis-pdf`
(`spike/krilla`), não em fork, para **medir**, não assumir, se e como o
krilla poderia substituir ou complementar o motor atual. Plano de
medição em `docs/architecture/spike-krilla-plano.md`. Esta entrada não
decide adoção; decide apenas correr o spike e como o correr. A decisão
de adoção ou rejeição entra como ADR próprio (`aceite` ou `rejeitada`)
quando houver resultados.

### Alternativas rejeitadas
Fork do `normordis-pdf`: fragmentaria o histórico e dificultaria
comparar sob as mesmas condições de CI/veraPDF. Adotar sem spike: o
motor cobre só UA-1 e adotá-lo sem medir seria regressão não
verificada no diferenciador do projeto. Não avaliar: o
`normordis-kernel` já usa krilla transitivamente via `render-typst`,
ignorar essa sobreposição desperdiça informação já disponível no
ecossistema NORMORDIS.

### Consequências
Nenhuma alteração de código de produção até haver resultados. Se o
spike concluir por adoção (parcial ou total), contribuir PDF/UA-2
upstream ao krilla é o entregável identificado como mais forte para a
candidatura NLnet/Restack (infraestrutura partilhada, não motor
fechado) — decisão de âmbito da candidatura fica para essa altura. Se
concluir por rejeição, a branch `spike/krilla` é arquivada com os
números medidos, para a avaliação não se repetir sem informação nova.

### Atualização — 2026-09-04
Esclarecimento de contexto que faltava na entrada original: o
`normordis-pdf` **já produz PDF/UA-2 conforme com o motor próprio**,
verificado pelo veraPDF na CI (exemplo `13_accessibility`, check
"Validate PDF/UA-2") independentemente desta decisão. A UA-2 nunca foi
uma lacuna de produto que o krilla resolveria — a pergunta 1 do plano
de spike, respondida (PDF/A-4f via krilla passa veraPDF, 109 regras,
208 verificações, confirmado localmente e em CI), é sobre consolidação
de motor (menos código próprio, PDF/A-4 nativo), não sobre obter UA-2.

A oportunidade de contribuir PDF/UA-2 ao krilla upstream mantém-se, com
âmbito reposicionado: entra no roteiro do projeto como **trabalho
futuro**, não como pré-requisito da candidatura nem como spike a
concluir antes da submissão. O seu valor para a candidatura é
sobretudo o critério de impacto (infraestrutura partilhada com outros
consumidores do krilla, incluindo o Typst) — não o critério técnico,
que a UA-2 do motor próprio já demonstra por si. Decisão do
responsável: mantém-se no roteiro do projeto e é referida na
candidatura como trabalho futuro; não é executada nem estimada em
detalhe antes da submissão.

**Nota de âmbito, para evitar leitura errada:** o resultado da
pergunta 1 (`PdfA4Ua2` via krilla passa veraPDF) valida apenas o
spike, correndo em `spike/krilla`. Não valida, e não deve ser lido
como validando, a saída `PdfStandard::PdfA4Ua2` do motor de produção
(`pdf-writer`) publicado na 3.0.0 — esse continua sem validação
independente do perfil PDF/A-4f, conforme já indicado no README
("Independent validation of the PDF/A-4f profile is pending"). Os dois
motores são código distinto; um resultado positivo no krilla não é
evidência sobre o comportamento do `pdf-writer`.

## ADR-006 — MSRV declarado em 1.88, não 1.85

- **Data:** 2026-09-08
- **Estado:** aceite
- **Decisão de:** Carlos Canuto Costa
- **Proposta por:** agente (Claude Sonnet 5, Claude Code)
- **Origem:** preparação da release 3.0.1 (correções de documentação e
  metadados pedidas em resposta a uma verificação externa da 3.0.0);
  o job "msrv" novo no CI falhou logo na primeira execução

### Contexto
A primeira tentativa declarou `rust-version = "1.85"`, por ser o piso
teórico da edition 2024 (primeira stable a suportá-la). O job "msrv" do
CI, criado nesta mesma release para validar isso de facto, falhou de
imediato por duas causas independentes:

1. **Sintaxe da própria crate:** `if ... && let ... { }` (*let-chains*)
   é usado em 18 ocorrências, 9 ficheiros (`src/document.rs`,
   `src/fonts.rs`, `src/template/renderer.rs`, `src/elements/footer.rs`,
   `src/archive/integrity.rs`, `src/elements/paragraph.rs`,
   `src/elements/fixed_image.rs`, `src/elements/header.rs`,
   `src/backend/pdf_writer_backend.rs`). Let-chains estabilizou em
   Rust **1.88.0** (26 de junho de 2025), não em 1.85 —
   [rust-lang/rust#139951](https://github.com/rust-lang/rust/issues/139951).
   Também `u32::is_multiple_of` (estável desde 1.87) era usado em dois
   pontos, apanhado pelo lint `clippy::incompatible_msrv`.
2. **Dependências transitivas:** `image@0.25.10` exige 1.88.0;
   `icu_collections`, `icu_locale_core`, `icu_normalizer`,
   `icu_properties`, `icu_provider`, `idna_adapter` (via `url`/IDNA)
   exigem 1.86.

### Decisão
Declarar `rust-version = "1.88"` — o piso real, determinado pelo job de
CI, não um valor teórico. Não reescrever os let-chains para baixar o
MSRV nesta release: é uma alteração de lógica condicional em código de
produção, fora do âmbito "só documentação e metadados" da 3.0.1.

### Alternativas rejeitadas
Manter 1.85 e reescrever let-chains em 9 ficheiros para a forma aninhada
(`if let Some(x) = a { if cond { … } }`): tecnicamente possível, mas é
mudança de código de produção que exigiria testes e revisão próprios,
não cabe numa release de documentação. Não declarar `rust-version`
nesta release: rejeitado porque o job "msrv" já existe e já provou o
seu valor ao apanhar o erro na primeira execução — melhor declarar o
valor certo do que remover a validação.

### Consequências
Descer o MSRV abaixo de 1.88 fica como trabalho futuro, condicional a
reescrever os let-chains listados acima e a confirmar que as
dependências transitivas também o permitem nessa altura.
