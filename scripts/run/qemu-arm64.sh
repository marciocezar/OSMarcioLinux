#!/usr/bin/env bash
# Executa uma imagem ARM64 do OSMarcio Linux no QEMU (mesma máquina virtual
# que o UTM usa por baixo: "virt" + UEFI edk2 + VirtIO).
#
# uso:
#   qemu-arm64.sh <imagem.img> [--disk <disco.qcow2>] [--headless] [--selftest]
#                [--serial-log <arquivo>] [--mem MiB] [--cpus N]
#                [--gpu-headless] [--monitor <socket>]
#
#   --gpu-headless  placa de vídeo virtio-gpu + teclado/mouse USB sem janela
#                   (testes do desktop: a tela é lida pelo monitor do QEMU)
#   --monitor       monitor HMP num socket unix (screendump, sendkey)
#
# Por padrão cria (uma única vez) um disco persistente out/vm/<nome>.qcow2
# copiado da imagem; a imagem de release nunca é modificada e um disco
# existente nunca é sobrescrito.
source "$(dirname "$0")/../lib/common.sh"
need qemu-system-aarch64; need qemu-img

IMG=${1:?imagem}; shift
DISK=""; HEADLESS=0; GPUHL=0; MONSOCK=""; SELFTEST=""; SERIAL_LOG=""; MEM=4096; CPUS=4
while [[ $# -gt 0 ]]; do
  case $1 in
    --disk) DISK=$2; shift ;;
    --headless) HEADLESS=1 ;;
    --selftest) SELFTEST=$2; shift ;;
    --serial-log) SERIAL_LOG=$2; shift ;;
    --mem) MEM=$2; shift ;;
    --cpus) CPUS=$2; shift ;;
    --gpu-headless) GPUHL=1 ;;
    --monitor) MONSOCK=$2; shift ;;
    *) die "opção desconhecida: $1" ;;
  esac; shift
done
[[ -f $IMG ]] || die "imagem não encontrada: $IMG"

VMDIR="$OSM_ROOT/out/vm"; mkdir -p "$VMDIR"
name=$(basename "$IMG" .img)
if [[ -z $DISK ]]; then
  DISK="$VMDIR/$name.qcow2"
  if [[ ! -e $DISK ]]; then
    log "criando disco persistente $DISK (20 GiB virtuais)"
    qemu-img convert -O qcow2 "$IMG" "$DISK"
    qemu-img resize -q "$DISK" 20G
  fi
fi

# Firmware UEFI (edk2) — vem com o QEMU do Homebrew.
QEMU_SHARE="$(cd "$(dirname "$(command -v qemu-system-aarch64)")/../share/qemu" && pwd)"
CODE="$QEMU_SHARE/edk2-aarch64-code.fd"
[[ -f $CODE ]] || die "firmware UEFI não encontrado: $CODE"
VARS="${DISK%.*}-efivars.fd"
[[ -e $VARS ]] || { truncate -s 64M "$VARS"; }

# Aceleração: HVF (Apple Silicon) se disponível; senão emulação TCG.
ACCEL=(-accel tcg -cpu max)
if [[ $(uname -s) == Darwin && $(uname -m) == arm64 ]] && sysctl -n kern.hv_support 2>/dev/null | grep -q 1; then
  ACCEL=(-accel hvf -cpu host)
fi

args=(
  -machine virt,highmem=on "${ACCEL[@]}" -smp "$CPUS" -m "$MEM"
  -drive "if=pflash,format=raw,readonly=on,file=$CODE"
  -drive "if=pflash,format=raw,file=$VARS"
  -drive "if=none,id=hd0,format=qcow2,file=$DISK,discard=unmap"
  -device virtio-blk-pci,drive=hd0
  -nic user,model=virtio-net-pci
  -device virtio-rng-pci
)
if [[ $GPUHL == 1 ]]; then
  args+=(-device virtio-gpu-pci -device qemu-xhci -device usb-kbd -device usb-tablet -display none)
  [[ -n $MONSOCK ]] || args+=(-monitor none)
  if [[ -n $SERIAL_LOG ]]; then args+=(-serial "file:$SERIAL_LOG"); else args+=(-serial stdio); fi
elif [[ $HEADLESS == 1 ]]; then
  args+=(-display none -monitor none)
  if [[ -n $SERIAL_LOG ]]; then args+=(-serial "file:$SERIAL_LOG"); else args+=(-serial stdio); fi
else
  args+=(-device virtio-gpu-pci -device qemu-xhci -device usb-kbd -device usb-tablet -display cocoa -serial stdio)
fi
[[ -n $MONSOCK ]] && args+=(-monitor "unix:$MONSOCK,server,nowait")
[[ -n $SELFTEST ]] && args+=(-fw_cfg "name=opt/osmarcio/selftest,string=$SELFTEST")

log "QEMU: $(basename "$DISK") accel=${ACCEL[1]} mem=${MEM}M cpus=$CPUS"
exec qemu-system-aarch64 "${args[@]}"
