# Fontes incluídas — atribuição e licenciamento

Este diretório redistribui software de fontes de terceiros, **sem modificação**.
As licenças exigem que o aviso de copyright e o texto da licença acompanhem a
distribuição; é essa a função deste ficheiro e dos ficheiros `LICENSE-*` que o
acompanham.

Os dados abaixo foram lidos das tabelas `name` dos próprios ficheiros TTF, não
presumidos a partir do nome do ficheiro.

| Família | Versão | Copyright | Licença | Texto |
|---|---|---|---|---|
| Liberation Sans | 2.1.5 | Digitized data (c) 2010 Google Corporation; (c) 2012 Red Hat, Inc. | SIL Open Font License 1.1 | [`LICENSE-OFL-1.1.txt`](LICENSE-OFL-1.1.txt) |
| Liberation Serif | 1.07.4 | (c) 2007 Red Hat, Inc. | Liberation Fonts License (GPLv2 com exceção de embutimento) | [`LICENSE-Liberation-1.x.txt`](LICENSE-Liberation-1.x.txt), [`LICENSE-GPL-2.txt`](LICENSE-GPL-2.txt) |
| Liberation Mono | 1.07.4 | (c) 2007 Red Hat, Inc. | Liberation Fonts License (GPLv2 com exceção de embutimento) | [`LICENSE-Liberation-1.x.txt`](LICENSE-Liberation-1.x.txt), [`LICENSE-GPL-2.txt`](LICENSE-GPL-2.txt) |
| Libertinus Serif | 7.051 | (c) 2012-2024 The Libertinus Project Authors | SIL Open Font License 1.1 | [`LICENSE-OFL-1.1.txt`](LICENSE-OFL-1.1.txt) |

LIBERATION é uma marca registada da Red Hat, Inc. Os nomes de fonte reservados
declarados pelas licenças — Liberation, Arimo, Tinos, Cousine, Linux Libertine,
Biolinum, STIX Fonts — não são usados em obras derivadas, porque não existem
obras derivadas: as fontes são redistribuídas tal como recebidas.

## Relação com a licença do projeto

O código deste repositório está sob EUPL-1.2 (ver [`LICENSE`](../../LICENSE)).
As fontes mantêm as licenças próprias indicadas acima e não passam a estar sob
EUPL por serem redistribuídas em conjunto.

A exceção de embutimento da Liberation Fonts License existe precisamente para
este caso: embutir a fonte, ou porções não alteradas dela, num documento
produzido com ela não sujeita esse documento à GPL. É o que sucede nos PDF
gerados por esta biblioteca.

## Pendência conhecida

As famílias Liberation não estão todas na mesma geração: `Liberation Sans` está
na 2.1.5, sob OFL 1.1, enquanto `Liberation Serif` e `Liberation Mono` estão na
1.07.4, sob a licença anterior. A divergência é de origem, não intencional.

Alinhar as três na série 2.x colocaria todo o conjunto sob OFL 1.1 e eliminaria
a necessidade dos ficheiros GPL. Não foi feito aqui porque substituir binários
de fonte pode alterar a métrica e o resultado visual dos PDF gerados, o que
exige revalidação das fixtures de conformidade — decisão de projeto, não de
licenciamento.
