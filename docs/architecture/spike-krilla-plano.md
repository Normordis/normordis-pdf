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

Por iniciar — branch `spike/krilla` criada, sem código ainda.
