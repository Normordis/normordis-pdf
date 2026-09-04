# Convenção de registo de proveniência de IA (commits)

Este documento define como distinguir, no histórico git, contribuições
assistidas por IA de contribuições humanas diretas, conforme a secção
"Transparency & logging" da política de IA generativa da NLnet. Complementa
o `AI_USAGE.md` na raiz do repositório.

## Regra geral

- **Commit com código gerado ou assistido por IA de forma substantiva:** a
  linha de autor indica o modelo, e o corpo inclui o prompt (ou resumo fiel)
  e uma nota sobre o output.
- **Commit de revisão ou correção humana sobre esse código:** autoria humana
  normal; o corpo referencia o commit de origem.
- **Commit exclusivamente humano:** formato normal, sem menção a IA. Não é
  necessário assinalar a ausência de IA.

Considera-se "substantivo" o uso de IA que afeta materialmente o conteúdo do
commit: código de implementação, alterações de comportamento, lógica de
validação. Correções ortográficas, formatação ou pequenos ajustes sugeridos
pela ferramenta não exigem registo.

## Identificação do modelo

"Claude" ou "Claude Code" não chega. Cada commit assistido indica o **nome e
a versão exatos** do modelo que produziu o output, tal como aparecem na
sessão (por exemplo `Claude Sonnet 5`, `Claude Opus 4.8`, `Claude Fable
5.1`). Se numa sessão se alternou de modelo, indica-se o modelo que gerou a
parte substantiva do commit; se ambos contribuíram, listam-se os dois.

O Claude Code acrescenta automaticamente um trailer `Co-Authored-By` com o
modelo ativo. Antes de fazer commit, confirma-se que o trailer e a linha de
autor nomeiam o mesmo modelo; em caso de discrepância prevalece o que foi
efetivamente usado e corrige-se a mensagem.

## Formato: commit assistido por IA

```
Author: Carlos Canuto Costa with <Modelo Versão> (Claude Code) <carloscanutocosta@gmail.com>
Date:   <data>

<título curto do commit>

Prompt: <prompt usado, ou resumo fiel se o prompt for muito longo>
Output: (este commit) | <resumo do output, se editado antes do commit>
Revisão humana: <o que foi verificado ou alterado manualmente antes de aceitar>

Co-Authored-By: <Modelo Versão> <noreply@anthropic.com>
```

A linha de autor com "with <modelo>" segue o exemplo literal da política da
NLnet e é a marca principal; o trailer é redundante mas mantém-se.

Para definir a linha de autor num commit concreto (exemplo com Sonnet 5):

```
git commit --author="Carlos Canuto Costa with Claude Sonnet 5 (Claude Code) <carloscanutocosta@gmail.com>"
```

### Exemplo adaptado ao projeto

```
Author: Carlos Canuto Costa with Claude Sonnet 5 (Claude Code) <carloscanutocosta@gmail.com>
Date:   Thu Sep 10 2026

Gerar testes de estrutura de tags PDF/UA-2 para normordis-pdf

Prompt: Gerar testes unitários para validar a estrutura de tags
semânticos exigida por PDF/UA-2 secção 7.1, a partir dos casos
de teste já definidos em docs/pdfua2-cases.md.
Output: (este commit)
Revisão humana: casos verificados manualmente contra ISO 14289-2;
dois testes ajustados por não cobrirem tabelas aninhadas.

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
```

## Formato: commit humano a corrigir ou integrar output de IA

```
Author: Carlos Canuto Costa <carloscanutocosta@gmail.com>
Date:   <data>

Corrigir geração de metadados XMP em <hash do commit anterior>

Ajusta o output gerado por IA no commit referido: o schema XMP
omitia o namespace pdfuaid; adicionado manualmente após verificação
contra ISO 19005-3.
```

## Quando não é necessário registo por commit

Se a IA for usada apenas para testes ou documentação (não para código de
implementação), a política da NLnet considera suficiente a descrição geral
do `AI_USAGE.md`. O registo por commit continua a ser preferível quando for
prático.

## Âmbito temporal

Esta convenção aplica-se a partir de 2026-09-04, data de entrada em vigor do
`AI_USAGE.md`, a todo o trabalho novo, financiado ou não. Não se aplica
retroativamente ao histórico anterior.
