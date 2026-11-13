#!/usr/bin/env bash
# Inicia o OSMarcio Linux no aplicativo UTM (Apple Silicon, virtualização ARM64).
#
# 1ª execução: cria um disco qcow2 de 20 GiB a partir da IMG (a IMG nunca é
# alterada), cria a VM no UTM por AppleScript com a IMG como DISCO VirtIO
# (não como CD/ISO — esse foi o erro de boot original) e a inicia.
# Execuções seguintes: só inicia a VM existente. Discos existentes nunca são
# sobrescritos nem recriados.
#
# uso: utm.sh <imagem.img> [--no-start]
source "$(dirname "$0")/../lib/common.sh"
need qemu-img

IMG=${1:?imagem}; START=1; [[ ${2:-} == --no-start ]] && START=0
[[ -f $IMG ]] || die "imagem não encontrada: $IMG"
[[ $(uname -s) == Darwin && $(uname -m) == arm64 ]] || die "make run-utm é para macOS Apple Silicon"
UTMCTL=/Applications/UTM.app/Contents/MacOS/utmctl
[[ -x $UTMCTL ]] || die "UTM não encontrado em /Applications/UTM.app (https://mac.getutm.app)"

base=$(basename "$IMG" .img)                  # OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0
prof=$(echo "$base" | sed -E 's/.*AArch64-([A-Za-z]+)-v.*/\1/')
VM_NAME="OSMarcio Linux $OSM_VERSION $prof"

open -ga UTM; sleep 2
if "$UTMCTL" list 2>/dev/null | awk '{$1="";$2=""; sub(/^  /,""); print}' | grep -qxF "$VM_NAME"; then
  log "VM '$VM_NAME' já existe no UTM — iniciando (discos preservados)"
else
  QCOW="$OSM_ROOT/out/utm/$base.qcow2"
  mkdir -p "$(dirname "$QCOW")"
  if [[ ! -e $QCOW ]]; then
    log "criando disco de 20 GiB: ${QCOW#$OSM_ROOT/}"
    qemu-img convert -O qcow2 "$IMG" "$QCOW"
    qemu-img resize -q "$QCOW" 20G
  else
    log "reutilizando disco existente: ${QCOW#$OSM_ROOT/}"
  fi
  log "criando VM '$VM_NAME' no UTM (ARM64, virtualização, UEFI, 4 GiB, 4 CPUs)"
  osascript - "$VM_NAME" "$QCOW" <<'OSA' >/dev/null
on run argv
  set vmName to item 1 of argv
  set disk to POSIX file (item 2 of argv)
  tell application "UTM"
    make new virtual machine with properties {backend:qemu, configuration:{name:vmName, architecture:"aarch64", machine:"virt", memory:4096, cpu cores:4, hypervisor:true, uefi:true, drives:{{removable:false, interface:VirtIO, source:disk}}, network interfaces:{{hardware:"virtio-net-pci", mode:shared}}, displays:{{hardware:"virtio-gpu-pci", dynamic resolution:true}}}}
  end tell
end run
OSA
  log "VM criada. O UTM copiou o disco para dentro do pacote da VM; ${QCOW#$OSM_ROOT/} pode ser mantido como cópia de segurança."
fi

if [[ $START == 1 ]]; then
  "$UTMCTL" start "$VM_NAME"
  log "VM iniciada. Senha temporária: aparece na tela de login (clique no usuário 'osmarcio')."
fi
"$UTMCTL" status "$VM_NAME" | sed 's/^/[utm] estado: /'
