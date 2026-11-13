# shellcheck shell=bash
# Funções compartilhadas pelos scripts do OSMarcio Linux.
# Sem caminhos absolutos: tudo é relativo à raiz do repositório.

set -euo pipefail

OSM_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OSM_VERSION="$(tr -d '[:space:]' < "$OSM_ROOT/VERSION")"
OSM_BUILDER_IMAGE="osmarcio-linux-builder:trixie"
OSM_SUITE="${OSM_SUITE:-trixie}"
OSM_MIRROR="${OSM_MIRROR:-http://deb.debian.org/debian}"

log()  { printf '\033[1;34m[osmarcio]\033[0m %s\n' "$*" >&2; }
warn() { printf '\033[1;33m[aviso]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[erro]\033[0m %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "ferramenta ausente: $1 (rode 'make doctor')"; }

# Recusa sobrescrever um artefato já existente (regra: não sobrescrever imagens antigas).
refuse_overwrite() {
  if [[ -e "$1" ]]; then
    die "o arquivo já existe e não será sobrescrito: $1
      -> mova/renomeie-o manualmente, ou incremente VERSION."
  fi
}

# Executa um comando dentro do container de build, com o repositório em /work.
# --privileged é necessário para chroot+mount do mmdebstrap; o escopo é a VM
# do Docker Desktop, não o macOS/Windows.
in_builder() {
  need docker
  # O target Linux fica em out/target-linux-<arch>, separado do target/ do
  # host (macOS/Windows) — os builds nunca se misturam.
  docker run --rm --privileged \
    -v "$OSM_ROOT":/work -w /work \
    -v osmarcio-cargo-registry:/opt/cargo/registry \
    -e OSM_SUITE -e OSM_MIRROR \
    -e CARGO_TARGET_DIR="/work/out/target-linux-$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')" \
    -e CEF_PATH="/work/out/cef/linux-$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')" \
    "$OSM_BUILDER_IMAGE" "$@"
}
