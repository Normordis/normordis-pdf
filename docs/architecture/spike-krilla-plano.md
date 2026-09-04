# Plano do spike krilla

Referenciado por ADR-005 em [DECISIONS.md](DECISIONS.md).

## Objetivo

Medir se `krilla` 0.8.2 pode substituir ou complementar
pdf-writer+subsetter+rustybuzz como motor de conformidade PDF/A +
PDF/UA-2, com números verificáveis por veraPDF — não opinião.

## Perguntas que o spike responde

1. Um documento PDF/A-4f gerado via krilla passa `veraPDF --flavour 4f`?
2. Que superfície do motor atual (módulos/linhas em `src/backend`,
   `src/compliance`) deixaria de ser necessária?
3. Que parte do caminho crítico de conformidade ficaria dependente do
   krilla, e com que garantias de manutenção (cadência de releases,
   mantenedor, issues abertas sobre UA)?
4. Esforço estimado (dias-pessoa) para contribuir PDF/UA-2 upstream ao
   krilla, dado que já tem *tagging* e base PDF 2.0 (usada pelo A4)?
5. O krilla resolve, por si, o renderizador NDT 2.0.0 em falta (item 3b
   do `TODO.md`), ou é ortogonal a esse problema?

## Fora de âmbito

Substituir a decisão de migração de formatos (ADR-003) — este spike é
sobre o motor de *rendering*/conformidade, não sobre a fonte de
verdade dos formatos NDF/NDT/NCRTF.

## Critérios de saída

- **Vale a pena aprofundar** se (1) passa **e** o custo estimado de
  UA-2 upstream (4) for inferior ao de manter o motor próprio
  equivalente.
- **Rejeição** se (1) falhar, ou se (3) revelar risco de manutenção
  incompatível com um caminho crítico de conformidade legal.

## Estado

**Pergunta 1 — respondida em 2026-09-04, PASSOU.**
`tools/spike-krilla` gera um documento mínimo (uma página, um `draw_text`)
com `krilla::configure::{ConfigurationBuilder, Archival::A4F}` — API
confirmada por leitura direta de
`crates/krilla/src/configure/{mod,validate}.rs` no repositório do krilla,
não por documentação de segunda mão. Validado localmente com
`tools/verify-pdf --flavour 4f --pdfa-only`, veraPDF 1.30.2 (mesma versão
e instalação documentada em `.github/workflows/verapdf.yml`):

```
PDF/A-4F → PASSOU  (109 regras, 208 verificações)
```

**Confirmado de forma independente na CI** (PR #8, GitHub-hosted runner,
run `33875467029`): mesmo resultado exato — `PDF/A-4F → PASSOU (109
regras, 208 verificações)`. Pergunta 1 fechada com dois pontos de
medição concordantes, não uma afirmação isolada.

Nota lateral, não parte da pergunta 1: adicionar krilla ao workspace
atualizou `subsetter` de 0.2.3 para 0.2.6 no `Cargo.lock` partilhado
(dependência também usada pelo motor atual, mesma faixa semver `0.2`) —
sem efeito no motor atual, mas registado porque um workspace partilhado
significa que a dependência do spike já toca o lockfile de produção.

**Pergunta 4 — reenquadrada em 2026-09-04, deixa de condicionar o
spike.** Verificação adicional (independente da pergunta 1): o
`normordis-pdf` já produz PDF/UA-2 conforme com o motor próprio,
validado pelo veraPDF na CI (exemplo `13_accessibility`). A UA-2 nunca
foi uma lacuna de produto que o krilla resolveria — ver a adenda
"Atualização — 2026-09-04" ao ADR-005 em
[DECISIONS.md](DECISIONS.md). Consequência para este plano: o critério
de saída "vale a pena aprofundar se o custo de UA-2 upstream for
inferior ao do motor próprio" deixa de fazer sentido tal como estava
formulado — não há troca a fazer, porque o motor próprio já cumpre.
A contribuição upstream mantém-se no roteiro do projeto como trabalho
futuro (critério de impacto para a candidatura NLnet/Restack), sem
estimativa de esforço nem execução agora. A pergunta 4 fica **por
responder, sem prazo**.

**Perguntas 2, 3 e 5 — por responder, sem urgência identificada.** Não
têm ligação direta à candidatura (essa ligação, via a pergunta 4, foi
desfeita pelo reenquadramento acima); continuam relevantes para a
saúde do motor a prazo, mas sem prazo fixado.

**Spike em pausa a partir de 2026-09-04**, por decisão do responsável:
a questão que motivou a urgência (UA-2 valorizar ou não a candidatura)
está resolvida e registada; retomar quando houver motivação própria
para decidir consolidação de motor, não por pressão de calendário da
candidatura.
