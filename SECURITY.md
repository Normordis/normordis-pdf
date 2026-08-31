# Política de segurança

## Comunicação responsável

Não abra uma issue pública para comunicar uma vulnerabilidade, uma exposição de
dados ou uma chave acidentalmente publicada.

Use a opção **Report a vulnerability** no separador **Security** do repositório
GitHub e inclua:

- uma descrição clara do impacto;
- passos mínimos para reproduzir, ou uma prova de conceito não destrutiva;
- a versão do crate e o ficheiro ou função afetados; e
- uma forma segura de contacto para acompanhamento.

## Âmbito

`normordis-pdf` gera PDF a partir de conteúdo estruturado. São especialmente
relevantes:

- construção de PDF a partir de entrada não confiável — conteúdo NDT/NCRTF
  malformado que provoque pânico, consumo ilimitado de memória ou escrita fora
  do ficheiro de destino;
- resolução de recursos externos (imagens, fontes) a partir de caminhos
  controlados pela entrada;
- qualquer caminho que produza um PDF que declare conformidade PDF/A ou
  PDF/UA-2 sem a possuir, uma vez que a declaração pode ser usada como
  evidência.

Este repositório não é um serviço alojado e não recebe documentos reais nem
segredos operacionais.

## Tratamento

O maintainer confirma a receção, avalia a reprodução e coordena a divulgação.
Não publique detalhes antes de existir correção, mitigação ou decisão
documentada. Problemas que afetem uma versão publicada no crates.io são
registados no changelog e, quando aplicável, recebem um advisory GitHub.

## Regras para contribuidores

Nunca inclua tokens, palavras-passe, chaves privadas, certificados privados,
dados pessoais reais ou documentos operacionais. Exemplos e fixtures devem usar
dados sintéticos.

## Utilização de IA generativa

O desenvolvimento deste projeto segue a política de utilização de IA generativa
do ecossistema NORMORDIS, publicada em
[normordis-formats/AI_USAGE.md](https://github.com/carloscanutocosta/normordis-formats/blob/main/AI_USAGE.md).
Código de autenticação, credenciais e controlo de acesso não é aceite a partir
de geração assistida sem revisão linha a linha.
