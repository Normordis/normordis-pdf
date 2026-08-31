# Scripts

Auxiliares de compilação e publicação. Bash é a opção principal em Linux; o
equivalente PowerShell mantém a compatibilidade com Windows.

## Bash (Linux)

- `bash/build-release.sh` — compila a biblioteca e as ferramentas CLI em modo
  release.
- `bash/publish-crate.sh` — valida ou publica a crate no crates.io.

```bash
# Compilação release com cache fora do repositório
bash scripts/bash/build-release.sh \
  --cargo-target-dir /caminho/para/cache/normordis-pdf/linux-x86_64

# Cross-build Windows (requer target e linker Windows instalados)
bash scripts/bash/build-release.sh \
  --target x86_64-pc-windows-gnu \
  --cargo-target-dir /caminho/para/cache/normordis-pdf/windows-x86_64 \
  --out-dir dist/windows-x86_64

# Validar a publicação sem enviar nada
bash scripts/bash/publish-crate.sh --cargo-target-dir /caminho/para/cache/target

# Publicar, depois da validação e com credencial configurada por `cargo login`
bash scripts/bash/publish-crate.sh --publish --cargo-target-dir /caminho/para/cache/target
```

Cada build copia as CLI, a biblioteca C (`libnormordis_pdf.so`,
`normordis_pdf.dll` ou `libnormordis_pdf.dylib`) e `include/normordis_pdf.h`
para o diretório de saída. O sufixo é determinado pelo `--target`, não pelo
sistema onde o comando corre.

## PowerShell (Windows)

- `powershell/build-release.ps1`

```powershell
.\scripts\powershell\build-release.ps1 `
  -Target x86_64-pc-windows-msvc `
  -CargoTargetDir 'D:\BuildCache\normordis-pdf\windows-x86_64' `
  -OutDir 'dist\windows-x86_64'
```

## Nota sobre `target/`

`target/` é um artefacto local da máquina onde ocorre a compilação, nunca parte
do repositório. Cada execução recebe a sua localização por `--cargo-target-dir`
(Bash) ou `-CargoTargetDir` (PowerShell). Isto evita colisões entre projetos e
mantém o checkout limpo. Se a opção for omitida, os scripts usam uma cache
local do projeto (`~/.cache/normordis-pdf/target` em Bash,
`%LOCALAPPDATA%\NORMORDIS\normordis-pdf\target` em PowerShell).

Para correr a suite sem ocupar `/tmp`, use também um temporário fora do
repositório:

```bash
TMPDIR=/caminho/para/cache/tmp \
cargo test --workspace --target-dir /caminho/para/cache/target
```
