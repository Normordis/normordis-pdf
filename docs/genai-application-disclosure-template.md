# Prompt provenance log: candidatura NLnet / NGI

Este registo é exigido pela política de IA generativa da NLnet sempre que se
usa IA na preparação da candidatura (redação, tradução ou resumo), incluindo
interações durante a avaliação interativa. É submetido através do formulário
de candidatura (ver instruções em nlnet.nl/propose).

Preencher uma linha por interação relevante. "Output não editado" é o texto
tal como devolvido pelo modelo, antes de qualquer edição manual; se for
longo, junta-se como anexo ou secção separada e referencia-se na tabela.

| # | Data/Hora | Modelo | Secção da proposta afetada | Prompt (texto integral) | Output não editado (ou referência ao anexo) |
|---|-----------|--------|-----------------------------|-------------------------|---------------------------------------------|
| 1 |           |        |                             |                         |                                             |
| 2 |           |        |                             |                         |                                             |

## Instruções de preenchimento

1. **Modelo**: nome e versão exatos (por exemplo "Claude Sonnet 5",
   "Claude Opus 4.8", "Claude Fable 5.1"). "Claude" ou "Claude Code" sem
   versão não é aceitável. Se mudares de modelo a meio da preparação,
   regista cada interação com o modelo que a produziu.
2. **Secção da proposta afetada**: identifica claramente (por exemplo
   "Resumo do projeto", "Descrição técnica: normordis-pdf", "Orçamento e
   justificação").
3. **Prompt**: cola o prompt exato, não uma paráfrase. Se usaste várias
   iterações do mesmo prompt, regista cada iteração.
4. **Output não editado**: obrigatório. Se o texto final da proposta difere
   do output, isso é esperado e aceite. A NLnet quer perceber a diferença
   entre o que a IA produziu e o que ficou depois da edição e verificação
   humana, não impedir a edição.
5. Interações usadas apenas para pesquisa ou verificação de factos (não para
   gerar texto da proposta) não precisam de constar. A política cobre
   redação, tradução e sumarização, não pesquisa.

## Anexos

Colocar aqui, ou em ficheiros separados referenciados na tabela, os outputs
não editados que não cabem na célula.

## Nota sobre o uso do Claude Code neste contexto

Interações com o Claude Code (ou outro assistente) usadas para redigir ou
rever secções da candidatura contam para este registo, que é distinto do
registo de proveniência do código (`docs/ai-provenance.md`). São duas
obrigações separadas dentro da mesma política:

- **este registo** cobre a preparação do *texto da proposta*;
- **`docs/ai-provenance.md`** cobre o *desenvolvimento do projeto*, em curso
  publicamente e, mais tarde, financiado.
