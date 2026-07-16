#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
publish=false
skip_tests=false
cargo_target_dir="${XDG_CACHE_HOME:-$HOME/.cache}/normordis-pdf/publish-target"
tmp_dir="${TMPDIR:-${XDG_CACHE_HOME:-$HOME/.cache}/normordis-pdf/publish-tmp}"

usage() {
  cat <<'EOF'
Uso: publish-crate.sh [opções]

Por omissão executa a validação de publicação do crates.io sem enviar a crate.

  --publish                 publicar realmente no crates.io
  --skip-tests              não executar cargo test --workspace
  --cargo-target-dir PASTA  cache Cargo local, fora do repositório
  --tmp-dir PASTA           temporários de rustc/linker, fora de /tmp
  -h, --help                mostrar esta ajuda

A publicação real requer uma credencial previamente configurada com `cargo login`
ou a variável de ambiente CARGO_REGISTRY_TOKEN. Nunca coloques esse token no Git.
EOF
}

while (($#)); do
  case "$1" in
    --publish) publish=true ;;
    --skip-tests) skip_tests=true ;;
    --cargo-target-dir|--tmp-dir)
      option=$1
      (($# >= 2)) || { printf 'Falta a pasta após %s.\n' "$option" >&2; exit 2; }
      case "$option" in
        --cargo-target-dir) cargo_target_dir=$2 ;;
        --tmp-dir) tmp_dir=$2 ;;
      esac
      shift ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'Opção desconhecida: %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
  shift
done

cd "$repo_root"

command -v cargo >/dev/null 2>&1 || { printf 'O comando cargo é necessário.\n' >&2; exit 1; }
if ! git diff --quiet || ! git diff --cached --quiet; then
  printf 'O checkout tem alterações. Faz commit ou guarda-as antes de publicar.\n' >&2
  exit 1
fi

crate_name="$(awk -F'"' '/^\[package\]/{in_package=1; next} in_package && /^name = /{print $2; exit}' Cargo.toml)"
crate_version="$(awk -F'"' '/^\[package\]/{in_package=1; next} in_package && /^version = /{print $2; exit}' Cargo.toml)"
[[ -n "$crate_name" && -n "$crate_version" ]] || {
  printf 'Não foi possível obter o nome e versão da crate em Cargo.toml.\n' >&2
  exit 1
}

mkdir -p -- "$cargo_target_dir" "$tmp_dir"
export CARGO_TARGET_DIR="$cargo_target_dir"
export TMPDIR="$tmp_dir"

printf '>>> [CRATES.IO] %s v%s\n' "$crate_name" "$crate_version"
printf '>>> [CRATES.IO] Cache Cargo: %s\n>>> [CRATES.IO] Temporários: %s\n' "$cargo_target_dir" "$tmp_dir"

if ! $skip_tests; then
  printf '>>> [CRATES.IO] A executar testes da workspace...\n'
  cargo test --workspace
else
  printf '>>> [CRATES.IO] Testes ignorados (--skip-tests).\n'
fi

printf '>>> [CRATES.IO] A validar o pacote...\n'
cargo package --package "$crate_name" --no-verify

if ! $publish; then
  printf '>>> [CRATES.IO] Simulação de publicação (nenhum envio será feito)...\n'
  cargo publish --package "$crate_name" --dry-run
  printf '>>> [CRATES.IO] Validação concluída. Usa --publish para enviar a crate.\n'
  exit 0
fi

printf '>>> [CRATES.IO] A publicar %s v%s...\n' "$crate_name" "$crate_version"
cargo publish --package "$crate_name"
printf '>>> [CRATES.IO] Publicação concluída.\n'
