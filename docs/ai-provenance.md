# Convenção de registo de proveniência de IA

Complementa [AI_USAGE.md](../AI_USAGE.md). Esta convenção trata do desenvolvimento
do projeto, não do bloco `proveniencia_ia` dos documentos NDF.

## Autoria humana e proveniência explícita (2026-09-19)

A autoria Git é exclusivamente humana. Carlos Canuto Costa assume a autoria e
responsabilidade do projeto. IA é assistência/ferramenta: não deve constar dos
campos de autor ou committer, de trailers de coautoria, nem de mecanismos que
atribuam a modelos o estatuto de contributor GitHub. Desativar a coautoria
automática da ferramenta antes de criar commits.

Toda a assistência substantiva, incluindo documentação e testes, é declarada
no corpo do commit. Identificar o modelo exato quando conhecido; quando não
houver evidência da versão, declarar essa limitação sem adivinhar.

O corpo contém `AI assistance:`, `Human decision:`, `AI contribution:` e
`Human review:`. A decisão identifica o requisito, pedido, issue, ADR ou critério
aprovado pelo responsável. A contribuição descreve o trabalho efetivo da IA.
`Human review` é preenchido exclusivamente pelo responsável humano, com as
verificações realmente feitas antes da aceitação. Não criar commits novos com
placeholders, revisão vazia ou uma alegação de revisão escrita pelo agente.
Sem revisão humana fornecida, conservar as alterações como diff para revisão.

Alterações normativas, schemas e critérios de conformidade precisam de origem
humana identificável. A IA pode ajudar a redigir e a propagar uma decisão humana
para schemas, exemplos, fixtures, índices e testes; não define autonomamente
requisitos. A conformidade não pode depender apenas da mesma sessão/modelo que
implementou o comportamento: exige critérios derivados da especificação e
verificação independente, incluindo validadores externos quando disponíveis.

Não apagar nem falsificar proveniência. A normalização histórica autorizada
preserva os registos originais em backup e os SHA num mapa de auditoria.
Declarações históricas de revisão são preservadas como declarações, sem nova
certificação. Uma lacuna histórica é registada como `Human review status`, nunca
convertida numa revisão fictícia; isto não permite aceitar novos commits sem
revisão. Uma revisão posterior só pode ser atestada pelo humano que a efetuou,
com data, âmbito e limitações. Verificações do agente são identificadas como
automatizadas e não contam como revisão humana.

Quando a divisão histórica não puder ser estabelecida, registar:

> Human provenance note:
> The precise division between human and AI contribution cannot be reconstructed reliably from the available record.

A ausência de declaração num commit histórico não demonstra ausência de IA.

## Estrutura da mensagem

Após o título, usar os quatro campos seguintes no corpo (sem trailers de
coautoria de IA):

- `AI assistance:` — modelo efetivamente usado, com versão quando conhecida.
- `Human decision:` — decisão/requisito e origem humana identificável.
- `AI contribution:` — redação, implementação, propagação, testes ou revisão
  assistida efetivamente realizados.
- `Human review:` — relato concreto fornecido pelo responsável humano.

Preservar prompts e notas de output quando disponíveis. Nunca inventar um
prompt, uma decisão ou uma revisão para completar a estrutura. Esta descrição
não é uma mensagem pronta a commitar.

## Auditoria

Localizar assistência declarada com `git log --all --grep="AI assistance:"`.
Consultar também os registos históricos de prompts e os mapas de normalização.
A convenção nova aplica-se ao trabalho futuro; a normalização histórica corrige
metadados mediante evidência, sem alterar requisitos ou resultados funcionais.
