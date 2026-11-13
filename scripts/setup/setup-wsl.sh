#!/usr/bin/env bash
# OSMarcio Linux — preparação do WSL2 (Ubuntu/Debian).
#
#   setup-wsl.sh --check   só diagnostica (make doctor-wsl)
#   setup-wsl.sh           diagnostica e OFERECE instalar via apt (pergunta antes)
#
# O repositório fica em /mnt/c/Users/<usuário>/_code/OSMarcioLinux — a MESMA
# pasta física usada pelo Windows. Os builds Linux vão para out/ (separado de
# qualquer build Windows), e o target Rust Linux para out/target-linux-<arch>.
source "$(dirname "$0")/../lib/common.sh"

CHECK_ONLY=0; [[ ${1:-} == --check ]] && CHECK_ONLY=1
apt_missing=(); ok=0; bad=0
row() { local c; case $1 in OK) c=32;; FALTA) c=31;; AVISO) c=33;; esac; printf '  \033[%sm%-7s\033[0m %-24s %s\n' "$c" "$1" "$2" "$3"; }
req() { if eval "$2" >/dev/null 2>&1; then row OK "$1" "$(eval "$2" 2>/dev/null | head -1)"; ok=$((ok+1)); else row FALTA "$1" "$3"; bad=$((bad+1)); [[ -n ${4:-} ]] && apt_missing+=("$4"); fi; }

echo "OSMarcio Linux $OSM_VERSION — diagnóstico do WSL2"
if grep -qi microsoft /proc/version 2>/dev/null; then row OK "WSL" "$(uname -r)"; else row AVISO "WSL" "não detectado (Linux nativo também funciona)"; fi
row OK "Distribuição" "$(. /etc/os-release; echo "$PRETTY_NAME")"
row OK "Arquitetura" "$(uname -m)"
req "APT"          "apt-get --version"            "use uma distro Debian/Ubuntu no WSL2"
req "Git"          "git --version"                "sudo apt install git" git
req "Make"         "make --version"               "sudo apt install make" make
req "Python 3"     "python3 --version"            "sudo apt install python3" python3
req "shasum"       "shasum --version"             "sudo apt install perl" perl
req "QEMU x86_64"  "qemu-system-x86_64 --version" "sudo apt install qemu-system-x86" qemu-system-x86
req "qemu-img"     "qemu-img --version"           "sudo apt install qemu-utils" qemu-utils
req "OVMF (UEFI)"  "ls /usr/share/OVMF/OVMF_CODE*.fd" "sudo apt install ovmf" ovmf
req "Rust (rustup)" "rustup --version"            "https://rustup.rs (opcional no host: os apps compilam no container)"
req "Docker"       "docker info --format '{{.ServerVersion}}'" "Docker Desktop com integração WSL2, ou docker.io no WSL"

# Bits de execução em /mnt/c exigem a opção "metadata" do automount.
if [[ $OSM_ROOT == /mnt/* ]]; then
  if mount | grep -E " on /mnt/c .*metadata" >/dev/null; then row OK "/mnt/c metadata" "permissões Unix preservadas"
  else row AVISO "/mnt/c metadata" "adicione em /etc/wsl.conf: [automount] options = \"metadata\" e rode 'wsl --shutdown'"; fi
fi

echo; echo "Resumo: $ok OK, $bad faltando."
if [[ $CHECK_ONLY == 1 ]]; then [[ $bad -eq 0 ]]; exit $?; fi
if [[ ${#apt_missing[@]} -gt 0 ]]; then
  echo "Pacotes APT faltando: ${apt_missing[*]}"
  read -r -p "Instalar com 'sudo apt-get install' agora? [s/N] " ans
  [[ $ans =~ ^[sS]$ ]] && sudo apt-get update && sudo apt-get install -y "${apt_missing[@]}"
fi
echo "Pronto. Rode 'make doctor-wsl' para conferir."
