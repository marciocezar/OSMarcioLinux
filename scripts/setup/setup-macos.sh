#!/usr/bin/env bash
# OSMarcio Linux — preparação do macOS Apple Silicon.
#
#   setup-macos.sh --check   só diagnostica (make doctor-mac)
#   setup-macos.sh           diagnostica e OFERECE instalar o que falta,
#                            perguntando item a item (nada é instalado em silêncio)
#
# Por que Docker?  mmdebstrap precisa de um kernel Linux (chroot, mounts).
# O macOS não tem; o container roda na VM Linux do Docker Desktop.
source "$(dirname "$0")/../lib/common.sh"

CHECK_ONLY=0; [[ ${1:-} == --check ]] && CHECK_ONLY=1
missing=(); ok=0; bad=0

row() { # row <estado> <nome> <detalhe>
  local c; case $1 in OK) c=32;; FALTA) c=31;; OPCIONAL) c=33;; *) c=0;; esac
  printf '  \033[%sm%-9s\033[0m %-26s %s\n' "$c" "$1" "$2" "$3"
}
req() { # req <nome> <comando-teste> <dica-instalação> [brew-formula]
  if eval "$2" >/dev/null 2>&1; then row OK "$1" "$(eval "$2" 2>/dev/null | head -1)"; ok=$((ok+1))
  else row FALTA "$1" "$3"; bad=$((bad+1)); [[ -n ${4:-} ]] && missing+=("$4"); fi
}
opt() {
  if eval "$2" >/dev/null 2>&1; then row OK "$1" "$(eval "$2" 2>/dev/null | head -1)"
  else row OPCIONAL "$1" "$3"; fi
}

echo "OSMarcio Linux $OSM_VERSION — diagnóstico do macOS"
[[ $(uname -s) == Darwin ]] || die "este script é para macOS"
row "$([[ $(uname -m) == arm64 ]] && echo OK || echo FALTA)" "Apple Silicon" "$(uname -m), macOS $(sw_vers -productVersion)"
req "Homebrew"       "brew --version"                 "https://brew.sh (instalação manual)"
req "Git"            "git --version"                  "xcode-select --install"
req "GNU Make"       "make --version"                 "xcode-select --install"
req "Python 3"       "python3 --version"              "xcode-select --install"
req "shasum"         "shasum --version"               "faz parte do macOS"
req "Rust (rustup)"  "rustup --version"               "https://rustup.rs (instalação manual)"
req "Docker CLI"     "docker --version"               "Docker Desktop: https://docs.docker.com/desktop/setup/install/mac-install/"
req "Docker rodando" "docker info --format 'servidor {{.ServerVersion}} ({{.Architecture}})'" "abra o Docker Desktop"
req "QEMU aarch64"   "qemu-system-aarch64 --version"  "brew install qemu" qemu
req "qemu-img"       "qemu-img --version"             "brew install qemu" qemu
req "UEFI edk2 ARM64" "ls \"\$(dirname \"\$(command -v qemu-system-aarch64)\")/../share/qemu/edk2-aarch64-code.fd\"" "vem com o QEMU do Homebrew"
req "HVF (Hypervisor)" "sysctl -n kern.hv_support | grep -qx 1 && echo disponível" "Hypervisor.framework indisponível"
opt "UTM"            "defaults read /Applications/UTM.app/Contents/Info.plist CFBundleShortVersionString | sed 's/^/UTM /'" "https://mac.getutm.app (opcional; o QEMU já basta para testes)"
opt "utmctl (make run-utm)" "/Applications/UTM.app/Contents/MacOS/utmctl version" "vem dentro do UTM.app"
opt "Imagem UTM desktop" "ls images/utm/OSMarcioLinux-UTM-AArch64-Desktop-v$OSM_VERSION.img" "make image-utm"
opt "Builder image"  "docker image inspect $OSM_BUILDER_IMAGE --format '{{.Id}}' | cut -c1-19" "make builder"
opt "draw.io"        "ls -d /Applications/draw.io.app" "brew install --cask drawio (para exportar PNG dos diagramas)"

echo; echo "Resumo: $ok OK, $bad faltando."
if [[ $CHECK_ONLY == 1 ]]; then [[ $bad -eq 0 ]]; exit $?; fi

# Instalação assistida — somente fórmulas Homebrew, sempre com confirmação.
mapfile_unique() { printf '%s\n' "$@" | sort -u; }
for f in $(mapfile_unique "${missing[@]:-}"); do
  [[ -z $f ]] && continue
  read -r -p "Instalar '$f' com Homebrew agora? [s/N] " ans
  [[ $ans =~ ^[sS]$ ]] && brew install "$f" || echo "  pulado: $f"
done
if docker info >/dev/null 2>&1 && ! docker image inspect "$OSM_BUILDER_IMAGE" >/dev/null 2>&1; then
  read -r -p "Construir a imagem Docker de build (~2 GB) agora? [s/N] " ans
  [[ $ans =~ ^[sS]$ ]] && "$OSM_ROOT/scripts/build/builder-image.sh"
fi
echo "Pronto. Rode 'make doctor-mac' para conferir."
