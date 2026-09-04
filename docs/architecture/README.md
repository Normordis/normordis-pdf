# docs/architecture

Sede das decisões de arquitetura do `normordis-pdf`. O código diz *o que*
o sistema faz; este diretório diz *porquê* e *quem decidiu*.

## Conteúdo

| Ficheiro | Função |
|---|---|
| `DECISIONS.md` | Registo cronológico de decisões de arquitetura, formato, norma aplicável e dependência. Uma entrada por decisão. |
| `spike-krilla-plano.md` | Plano de medição do ADR-005 — perguntas, critérios de saída e estado do spike krilla. |

Documentos de arquitetura mais longos (diagramas, análises de alternativas,
planos de migração) entram aqui como ficheiros próprios e são referenciados
a partir da entrada correspondente em `DECISIONS.md`.

## Regras

- **Quem decide é o responsável humano do projeto.** Um agente de IA pode
  propor; a entrada regista a proposta e a aceitação separadamente. Ver
  `AGENTS.md`, secção 12.2, e `AI_USAGE.md`.
- Uma decisão entra no registo no mesmo conjunto de alterações que a
  implementa. Não se implementa primeiro e regista depois.
- Entradas não se apagam nem se reescrevem. Uma decisão revertida ou
  substituída recebe uma entrada nova que cita a anterior; a anterior
  passa ao estado `substituída`.
- O `CHANGELOG.md` regista o que mudou em cada versão; este registo guarda
  a razão. Não duplicar o conteúdo: citar.

## Formato de cada entrada

```
## ADR-NNN — <título curto>

- **Data:** AAAA-MM-DD
- **Estado:** aceite | substituída por ADR-MMM | rejeitada
- **Decisão de:** <responsável>
- **Proposta por:** <responsável | agente (modelo e versão)>
- **Origem:** <issue, achado, pedido, norma ou commit que motivou>

### Contexto
Qual era o problema ou a escolha em aberto.

### Decisão
O que ficou decidido, em termos verificáveis.

### Alternativas rejeitadas
O que foi considerado e porque não ficou. Omitir se a escolha for óbvia.

### Consequências
O que muda para o código, os formatos, a compatibilidade ou o processo.
```
