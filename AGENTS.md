# AGENTS.md — orientações para este repositório

Instruções locais para agentes que trabalhem neste repositório Rust.

## 1. Linguagem e comunicação

- Responder ao utilizador em pt-PT.
- Escrever documentação funcional, técnica e comentários em pt-PT.
- O código deve ter comentários claros e concisos; aplicar boas práticas de documentação de código.
- Manter identificadores de código em inglês, salvo quando o domínio exigir outra convenção.
- Ser direto sobre lacunas reais, riscos arquiteturais e limitações atuais.

## 2. Enquadramento do repositório

Este repositório é um crate Rust autónomo para um renderizador PDF reutilizável por projetos institucionais.

Inclui:

- implementação core do renderer PDF (`src/`);
- suporte a formatos NDT, NDF e NCRTF;
- ferramentas auxiliares em `tools/`;
- scripts de suporte em `scripts/`.

Objetivos principais:

- extrair capacidades comuns para crates reutilizáveis;
- manter aplicações leves, compostas e focadas no fluxo de uso;
- evitar que regras específicas de uma app contaminem bibliotecas transversais.

## 3. Ordem mínima de leitura

Antes de propor ou implementar alterações, ler apenas o mínimo relevante, nesta ordem:

1. `README.md` da raiz.
2. `MANUAL.md` do crate afetado (se existir) ou `README.md` do crate.
3. Testes relevantes do crate ou do fluxo afetado.
4. Implementação atual.

Se a alteração envolver comportamento, persistência, shape de dados, API pública ou wiring entre crates, a leitura do `MANUAL.md` e dos testes existentes é obrigatória.

## 4. Regras de arquitetura

- Aplicar princípios de clean code, clean architecture, DDD e SOLID quando fizer sentido.
- Preferir composição em vez de acoplamento rígido.
- Separar claramente lógica de domínio, persistência, renderização e interface CLI.
- Valorizar simplicidade: resolver a necessidade real com a menor mudança possível.

## 5. Convenções por área

- `src/`: código core deve ser robusto, tipado e compatível com versões menores. Evitar hacks de conveniência.
- `tools/`: CLI e utilitários devem ser claros, com UX de linha de comando consistente e erros legíveis.
- `examples/`: demonstrar uso real da API, sem lógica de negócios adicional.
- `tests/`: preferir testes de comportamento e integração do que apenas testes de implementação.
- `scripts/`: manter idempotência, mensagens claras e opções de simulação (`dry run` / `WhatIf`).

## 6. Documentação obrigatória

**Regra fundamental: a documentação é parte da alteração, não um passo opcional.**  
Nenhuma alteração de código está concluída sem a documentação correspondente atualizada no mesmo conjunto de mudanças.

O que atualizar em cada alteração:

| Documento | Quando atualizar |
|---|---|
| `CHANGELOG.md` | **Sempre** — qualquer adição, alteração ou remoção de API pública, comportamento ou formato |
| `README.md` | Quando muda a API de superfície, os exemplos, as features ou a instalação |
| `MANUAL.md` | Quando muda contrato, invariantes, limites, integração, comportamento observável ou API pública |
| `MANUAL.en.md` | Quando a secção equivalente do `MANUAL.md` muda |

Regras adicionais:

- `README.md` deve ser curto e orientado a objetivo/uso.
- `MANUAL.md` deve refletir o contrato atual, não a intenção ou o estado anterior.
- Não usar `MANUAL.md` como changelog — para isso existe o `CHANGELOG.md`.
- Se uma alteração mudar persistência, erros, workflow ou API pública, atualizar o `MANUAL.md` é **obrigatório**.
- Se encontrares documentação desalinhada da implementação, corrigir no mesmo conjunto de mudanças.

## 7. Regras de implementação

- Não programar contra suposições quando o contrato atual puder ser lido primeiro.
- Não deixar `TODO`, placeholders, pseudocódigo ou APIs falsas em código que se pretende ativo.
- Favorecer tipos explícitos, erros tipados e testes pequenos focados no comportamento.
- Preservar compatibilidade razoável nos crates transversais, salvo se a mudança for deliberada e documentada.
- Se encontrares um desvio entre documentação e implementação, corrigir ou sinalizar explicitamente.

## 8. Convenções de qualidade

- Correr testes relevantes após alterações, idealmente `cargo test` do crate afetado e do fluxo impactado.
- Quando o pedido toca integração transversal, validar com `cargo test --workspace`.
- Não introduzir estado global mutável sem necessidade clara.
- Não introduzir dependência de UI/framework visual em crates de infraestrutura.
- Quando uma abstração existe apenas em memória mas o projeto sugere persistência real, sinalizar isso explicitamente.

## 9. Raciocínio antes de alterar

Antes de fazer mudanças não triviais, explicitar de forma curta:

- qual crate é dono da alteração;
- se a necessidade é transversal ou específica da app;
- qual contrato ou testes são a referência principal;
- qual documentação precisa ser atualizada.

## 10. Fluxo de entrega

1. Ler o contexto mínimo relevante.
2. Resumir a abordagem estrutural.
3. Implementar.
4. **Atualizar documentação** — `CHANGELOG.md` sempre; `README.md`, `MANUAL.md`, `MANUAL.en.md` conforme o impacto.
5. Executar testes/checks adequados.
6. Entregar com resumo curto, riscos e validação.
7. **Preparar a mensagem de commit** conforme `docs/ai-provenance.md`, com o campo `Revisão humana:` por preencher pelo responsável (ver secção 12).

## 11. Checklist rápida por módulo

Quando alterares um crate ou app:

- Ler `README.md` e `MANUAL.md` relevantes antes de editar.
- Confirmar se a mudança é transversal ou específica da app.
- Verificar impacto em comportamento, persistência, wiring, erros e testes.
- **Atualizar `CHANGELOG.md`** — obrigatório em qualquer alteração de API, comportamento ou formato.
- Atualizar `README.md`, `MANUAL.md` e `MANUAL.en.md` quando o contrato ou a API de superfície mudou.
- Atualizar testes no mesmo conjunto de alterações.
- Garantir que a app host continua fina e as libs continuam reutilizáveis.
- Confirmar que a alteração tem uma origem humana identificável (especificação, decisão registada ou pedido explícito) e que a mensagem de commit a cita (secção 12).

## 12. Proveniência de IA e contribuição humana

Este repositório segue o `AI_USAGE.md` e o `docs/ai-provenance.md`. A regra
de fundo é que **nenhum entregável pode ser output puramente gerado por IA**:
a especificação, as decisões de arquitetura, os critérios de aceitação e a
revisão final são sempre do responsável humano, e essa contribuição tem de
ficar visível no repositório, não apenas ser verdadeira. Os agentes devem
garantir as condições seguintes em toda a alteração.

### 12.1 Origem humana obrigatória

- Não iniciar implementação sem uma origem humana identificável: uma
  especificação (`normordis-formats`, `MANUAL.md`, issue), uma decisão
  registada ou um pedido explícito do responsável.
- Se o pedido implicar uma decisão de arquitetura, de formato, de
  dependência ou de segurança que ainda não foi tomada, **parar e
  apresentar opções**; não decidir por omissão. A decisão é registada
  quando tomada (secção 12.2) e só depois se implementa.
- Citar essa origem na mensagem de commit (campo `Prompt:` ou corpo).

### 12.2 Registo de decisões

- Toda a decisão de arquitetura, de formato, de norma aplicável ou de
  dependência tomada pelo responsável durante uma sessão é escrita no
  documento adequado no mesmo conjunto de alterações (`MANUAL.md`,
  `README.md`, `CHANGELOG.md` ou `docs/architecture/`), identificada como
  decisão e não como descrição de código.
- Quando o agente propõe e o responsável aceita, o registo diz que foi
  proposta pelo agente e aceite pelo responsável, com a justificação dada.
- Alternativas rejeitadas e o motivo ficam registadas quando a escolha não
  for óbvia.

### 12.3 Mensagem de commit

- Preparar sempre a mensagem no formato de `docs/ai-provenance.md`: linha de
  autor com nome e versão exatos do modelo, `Prompt:`, `Output:` e
  `Revisão humana:`.
- O campo `Revisão humana:` **nunca é preenchido pelo agente**. Fica com o
  texto `<a preencher pelo responsável>` e é o responsável que o completa
  antes do commit, descrevendo o que verificou ou alterou. Um commit
  assistido sem esse campo preenchido não deve ser feito.
- Não fazer commit sem confirmação explícita do responsável.

### 12.4 Testes de conformidade independentes

- Testes que validem conformidade com normas (PDF/A, PDF/UA, assinaturas,
  schemas NDF/NDT/NCRTF) não são escritos na mesma sessão que produziu a
  implementação a testar. Se a sessão implementou a funcionalidade, deixa
  os casos de teste descritos em linguagem natural para uma sessão separada
  ou para o responsável, citando a cláusula da norma.
- Onde exista validador externo (veraPDF, validadores ETSI), o teste do
  projeto invoca-o ou reproduz o seu critério; não o substitui.

### 12.5 O que o agente não faz

- Não altera especificações de formato, critérios de aceitação nem
  requisitos normativos por iniciativa própria; sinaliza e espera.
- Não apresenta código gerado como revisto; a revisão é do responsável.
- Não remove nem reescreve registos de proveniência anteriores.

