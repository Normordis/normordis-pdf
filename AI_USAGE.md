# Política de Utilização de IA Generativa

**Projeto:** NORMORDIS e componentes (`normordis-pdf`, `normordis-formats`, `normordis-odf`, `normordis-viewer`, `normordis-cloud-sync`)
**Em vigor desde:** 2026-09-04
**Responsável pelo projeto:** Carlos Canuto Costa
**Licença do projeto:** EUPL-1.2

Este documento responde à política de IA generativa da NLnet (v1.1, em vigor
desde 2026-01-26). Está escrito de forma a servir também como resposta pronta
a perguntas da equipa de avaliação sobre a extensão do uso de IA no projeto.

## 0. O que este documento não é

O NORMORDIS e os seus componentes **não são sistemas de IA nem incorporam
modelos de IA em tempo de execução**. A IA generativa é usada apenas como
ferramenta de desenvolvimento, à semelhança de um editor ou de um compilador.
O software produzido é determinístico e não depende de nenhum serviço de IA
para funcionar. Por isso, o Regulamento (UE) 2024/1689 (Regulamento da
Inteligência Artificial) não se aplica ao produto; este documento trata
apenas da transparência sobre o processo de desenvolvimento.

## 1. Postura geral

Este projeto utiliza assistentes de código com IA generativa como **auxiliar de
implementação sob controlo arquitetural humano**. A ferramenta em uso é o
Claude Code (Anthropic). Os modelos usados variam por sessão: habitualmente
Claude Sonnet (versão 5 à data de entrada em vigor), pontualmente Claude
Fable 5.1 ou Claude Opus. "Claude" sem versão não é um registo válido: o
**nome e a versão exatos do modelo** são registados em cada commit
assistido, conforme `docs/ai-provenance.md`, e em cada linha do registo de
candidatura.

A IA não substitui a responsabilidade humana em nenhuma fase de decisão.

Resumo operacional: *AI-assisted development under human architectural
control, with documented provenance, mandatory review, automated conformance
testing, reproducible builds and human accountability for every deliverable.*

## 2. O que é, e o que não é, delegado à IA

**Permanece exclusivamente sob responsabilidade humana:**

- especificação e desenho dos formatos (NDF, NDT, NCRTF, `.ndfpkg` e todos os
  schemas associados, publicados em `normordis-formats`);
- decisões de arquitetura (estrutura dos crates, escolha de dependências,
  modelo de segurança, superfície de API pública);
- requisitos de conformidade com normas (PDF/A, PDF/UA, CAdES-B-LTA,
  PAdES-B-LTA, ISO, ETSI e RFC aplicáveis);
- critérios de aceitação e validação de cada entregável;
- revisão e aprovação final de qualquer código antes de merge ou release.

**Pode ser assistido por IA, sempre sob revisão humana:**

- geração de boilerplate e de código de implementação a partir de
  especificação já definida por humano;
- sugestões de refactoring;
- geração de testes (nunca como única fonte de verdade de conformidade; ver
  secção 4);
- documentação, tradução e resumo de texto técnico.

**Regra reforçada para código sensível.** Código que lide com credenciais,
autenticação, autorização, chaves ou material criptográfico (em particular em
`normordis-cloud-sync` e nos módulos de assinatura) só é aceite após revisão
humana linha a linha e nunca é integrado com base apenas na passagem de
testes.

## 3. Verificação de originalidade e licenciamento

Antes de integrar qualquer output de IA:

- confirma-se que o output não reproduz material protegido por direitos de
  autor incompatível com a EUPL-1.2;
- respeitam-se os termos de utilização da ferramenta (Anthropic) quanto à
  titularidade e originalidade dos outputs;
- nenhum output puramente gerado por IA é apresentado como entregável
  elegível para pagamento sem contribuição intelectual humana substancial,
  conforme exigido pela política da NLnet.

## 4. Testes de conformidade independentes

Os testes de conformidade (validação PDF/A e PDF/UA com veraPDF, validação de
assinaturas, validação de schema NDF/NDT/NCRTF) são escritos e mantidos com
base direta nas especificações. **Não são gerados pela mesma sessão de IA que
produziu a implementação a testar.** Isto evita que a suite de testes valide a
implementação através de um viés partilhado com o código gerado.

Sempre que existe um validador externo independente (veraPDF, validadores
ETSI), esse validador é a referência final, não os testes do próprio projeto.

## 5. Registo de proveniência

Ver `docs/ai-provenance.md`. Resumo:

- commits com contribuição substantiva de IA identificam o modelo na linha de
  autor e incluem o prompt (ou resumo fiel) e uma nota sobre o output;
- commits de revisão ou correção humana sobre output de IA são registados
  como autoria humana normal, com nota no corpo quando relevante;
- o Claude Code acrescenta automaticamente um trailer `Co-Authored-By` com o
  nome e versão do modelo ativo na sessão a todos os commits que cria; esse
  trailer é um marcador adicional e não substitui a linha de autor descrita
  acima, mas serve para confirmar qual o modelo que estava em uso.

## 6. Âmbito

Esta política aplica-se a partir da data indicada acima a todo o trabalho
iniciado depois dessa data, independentemente de estar ou não financiado.
Não é aplicada retroativamente ao histórico anterior do projeto, que foi
desenvolvido com assistência de IA sem registo formal por commit. Essa
ausência de registo é declarada aqui de forma transparente.

O texto das candidaturas a financiamento tem um registo próprio, distinto
deste; ver `docs/genai-application-disclosure-template.md`.

## 7. Contacto

Questões sobre o uso de IA neste projeto: Carlos Canuto Costa,
carloscanutocosta@gmail.com.
