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

## 11. Checklist rápida por módulo

Quando alterares um crate ou app:

- Ler `README.md` e `MANUAL.md` relevantes antes de editar.
- Confirmar se a mudança é transversal ou específica da app.
- Verificar impacto em comportamento, persistência, wiring, erros e testes.
- **Atualizar `CHANGELOG.md`** — obrigatório em qualquer alteração de API, comportamento ou formato.
- Atualizar `README.md`, `MANUAL.md` e `MANUAL.en.md` quando o contrato ou a API de superfície mudou.
- Atualizar testes no mesmo conjunto de alterações.
- Garantir que a app host continua fina e as libs continuam reutilizáveis.
