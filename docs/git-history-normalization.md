# Normalização histórica de autoria — 2026-09-19

Decisão humana: pedido explícito de Carlos Canuto Costa para autoria Git humana
com assistência de IA declarada, sem alterar conteúdo funcional ou inventar
revisões. A normalização foi executada com assistência de Codex (GPT-6; variante
exata não disponibilizada na sessão).

Cada árvore histórica foi preservada byte a byte (mesmo identificador Git).
Os modelos anteriormente indicados na autoria ou em trailers passaram para
`AI assistance`. Os prompts, notas de output e declarações históricas de revisão
foram preservados como registos, sem os certificar novamente. Versões de modelos
são as declaradas no histórico, não uma validação externa da sua identificação.

O [mapa completo](history-commit-map.tsv) permite resolver referências aos SHA
antigos, incluindo referências abreviadas únicas em documentação ou mensagens.
Não se reescreveram decisões normativas nem se inferiu ausência de IA nos
commits sem declaração. Quando a divisão do trabalho não é demonstrável,
registou-se explicitamente a incerteza. A autoria normalizada segue a atribuição
de responsabilidade autorizada pelo titular; não prova redação humana exclusiva.

Os bundles originais e o relatório operacional externo preservam metadados,
assinaturas, anotações de tags e os registos anteriores. Assinaturas de commits
reescritos deixam de ser válidas e não foram reproduzidas como se o fossem.
As datas originais foram mantidas. Tags anotadas conservam anotação e tagger,
mas o seu objeto muda para apontar para o commit normalizado.

A inspeção e os testes feitos pelo agente são verificações automatizadas,
não revisão humana. A declaração de revisão fornecida pelo responsável consta do commit
documental; não se acrescentam verificações humanas não declaradas.

## Lacunas auditadas

- `47504c8803c2809e42452256afede2617b1e6979` → `b1d24bdba216fb541f651012c40a13ccd15f7fe1`: O diff realinha a FFI com parse_ndt e DocumentBuilder::push_ndt. O prompt regista a decisão do responsável de corrigir a API. Não há revisão humana preenchida; não se confirmou retrospectivamente renderização NDT bem-sucedida.
- `aef323deb10eb734cf56a6feea5a4f28d41b06ee` → `10cf587794f36a5ecb4a7ae55c1928150bf29f54`: O diff acrescenta a importação de DocumentBuilder sob system-fonts. A descrição explica a correção, mas não permite atribuir uma decisão humana específica. Não há revisão humana preenchida.
- `b9a8c205a5fb1c0fe0e7cdda440a06dab20f3257` → `f1e80ae9bdd0b79cdbfed3024805361abc7a35af`: O diff marca três funções FFI como unsafe e documenta os contratos de ponteiros. O prompt regista confirmação e pedido do responsável. Não há revisão humana preenchida; testes não demonstram segurança de todos os chamadores externos.
- `7ac221492d61cb1fbc0b37b0c9a8aea46d745752` → `95db66ca12ade8c4b47b028134a50fd53d967f4a`: O diff renumera a versão para 3.0.0 e atualiza descrição e changelog. O prompt regista a decisão humana de publicação e de reservar 4.0.0. Os comandos históricos de validação não identificam quem os executou e não constituem revisão humana atestada.

## Novas assinaturas — 2026-09-20

Por pedido explícito do responsável, apenas os 9 commits deste
repositório que tinham assinatura foram novamente assinados com a chave SSH
do servidor. São assinaturas novas da normalização, não recriações das
assinaturas GitHub nem prova de revisão humana histórica. Nos commits
assinados, o committer passou para Carlos Canuto Costa, mantendo a data
histórica; a data desta operação é registada aqui. Mensagens, árvores e datas
foram preservadas, assim como anotações das tags. Os descendentes receberam
novos SHA sem assinatura adicional. A verificação criptográfica local passou;
o reconhecimento no GitHub depende de registar esta chave pública como
chave de assinatura. Não foi feito push ou registo de chave no GitHub.
