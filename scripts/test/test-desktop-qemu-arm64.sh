#!/usr/bin/env bash
# Teste REAL do desktop: inicializa a imagem ARM64 no QEMU/HVF com placa de
# vídeo virtio-gpu (a mesma do UTM), faz login pela tela gráfica do LightDM
# digitando como um usuário (monitor do QEMU: sendkey), cumpre a troca
# obrigatória de senha e coleta:
#   - linhas OSMTEST do autoteste interno (selftest.sh + selftest-desktop.sh);
#   - capturas de tela PNG do greeter e do desktop com Terminal e Browser.
#
# A imagem testada nunca é alterada (disco overlay descartável).
# uso: test-desktop-qemu-arm64.sh <imagem.img>
#   OSM_KEEP_VM=1  deixa a VM ligada no fim (inspeção via monitor: out/test-desktop-*/mon.sock)
source "$(dirname "$0")/../lib/common.sh"
need qemu-img; need python3

IMG=${1:?imagem}
[[ -f $IMG ]] || die "imagem não encontrada: $IMG"
IMG_ABS="$(cd "$(dirname "$IMG")" && pwd)/$(basename "$IMG")"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
TDIR="$OSM_ROOT/out/test-desktop-$STAMP"; mkdir -p "$TDIR"
REPDIR="$OSM_ROOT/reports/tests"; SHOTS="$REPDIR/screenshots"; mkdir -p "$SHOTS"
REPORT="$REPDIR/desktop-arm64-$STAMP.md"
SER="$TDIR/serial.log"
MON="$TDIR/mon.sock"               # socket unix: caminho < 104 bytes
M() { python3 "$OSM_ROOT/scripts/test/qemu-monitor.py" "$MON" "$@"; }
clean() { [[ -f $SER ]] || return 0; tr -d '\r' < "$SER" 2>/dev/null | sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g'; }
waitser() { # waitser <regex> <segundos>
  local t0=$SECONDS
  while (( SECONDS - t0 < $2 )); do clean | grep -aEq "$1" && return 0; kill -0 "$QPID" 2>/dev/null || return 1; sleep 2; done
  return 1
}
shot() { M screendump "$TDIR/$1.png"; cp "$TDIR/$1.png" "$SHOTS/desktop-arm64-$STAMP-$1.png"; log "captura: $1.png"; }

DISK="$TDIR/disk.qcow2"
qemu-img create -q -f qcow2 -F raw -b "$IMG_ABS" "$DISK" 20G

log "boot (QEMU/HVF, virtio-gpu, 4 GiB, 4 CPUs) — serial em ${SER#$OSM_ROOT/}"
"$OSM_ROOT/scripts/run/qemu-arm64.sh" "$IMG_ABS" --disk "$DISK" --gpu-headless --monitor "$MON" --qga "$TDIR/qga.sock" \
  --selftest "desktop$([[ ${OSM_KEEP_VM:-0} == 1 ]] || echo +poweroff)" --serial-log "$SER" --mem 4096 --cpus 4 >"$TDIR/qemu.out" 2>&1 &
QPID=$!
[[ ${OSM_KEEP_VM:-0} == 1 ]] || trap 'kill $QPID 2>/dev/null || true' EXIT

fails=0; steps=()
step() { steps+=("$1"); }
if waitser 'senha temporária: [A-Za-z0-9]+' 300; then
  PW=$(clean | grep -aEo -m1 'senha temporária: [A-Za-z0-9]+' | awk '{print $3}')
  step "PASS firmware → GRUB → kernel → systemd → firstboot (senha temporária gerada)"
else
  step "FAIL boot não chegou ao firstboot em 300 s"; fails=$((fails+1))
fi
if [[ -n ${PW:-} ]] && waitser 'OSMTEST (PASS|FAIL) display-manager' 600; then
  clean | grep -aq 'OSMTEST PASS display-manager' && step "PASS LightDM + Xorg + greeter ativos" \
    || { step "FAIL display manager não iniciou"; fails=$((fails+1)); }
  sleep 6; shot 01-greeter
  # Login gráfico + troca obrigatória de senha (fluxo PAM observado no greeter):
  # senha → "Changing password" (senha atual) → nova → repetir nova.
  # (sem "tr </dev/urandom | head": com pipefail o SIGPIPE abortaria o script)
  NEW=$(head -c 4096 /dev/urandom | LC_ALL=C tr -dc 'A-HJ-NP-Za-km-z2-9'); NEW=${NEW:0:16}
  for k in "$PW" "$PW" "$NEW" "$NEW"; do M type "$k"; M key ret; sleep 3; done
  if waitser 'OSMTEST INFO screenshot-browser' 600; then
    sleep 3; shot 02-browser
    # Prova de que a PÁGINA foi pintada (não só a moldura Rust): a área de
    # conteúdo vazia tem 1 cor só; com texto renderizado (antialiasing) tem
    # dezenas/centenas. O fundo pode ser claro ou escuro (prefers-color-scheme).
    st=$(python3 "$OSM_ROOT/scripts/test/qemu-monitor.py" - pngstat "$TDIR/02-browser.png" 60 160 1000 700)
    light=$(echo "$st" | sed -E 's/.*light=([0-9.]+).*/\1/'); ncol=$(echo "$st" | sed -E 's/.*colors=([0-9]+).*/\1/')
    if awk -v l="$light" -v c="$ncol" 'BEGIN{exit !(c >= 20)}'; then
      step "PASS browser-render conteúdo da página visível na tela ($st)"
    else
      step "FAIL browser-render área da página sem conteúdo ($st)"; fails=$((fails+1))
    fi
  fi
  if waitser 'OSMTEST END-DESKTOP' 600; then
    sleep 4; shot 03-desktop-terminal
    step "PASS login gráfico com troca obrigatória de senha"
  else
    shot 02-timeout; step "FAIL sessão gráfica não concluiu o autoteste em 600 s"; fails=$((fails+1))
  fi
fi
# Aguarda o desligamento pedido pelo autoteste.
if [[ ${OSM_KEEP_VM:-0} != 1 ]]; then
  for _ in $(seq 60); do kill -0 $QPID 2>/dev/null || break; sleep 1; done
  kill $QPID 2>/dev/null || true
fi

n_fail=$(clean | grep -ac '^OSMTEST FAIL' || true)
fails=$((fails + n_fail))
{
  echo "# Teste de desktop ARM64 — $(basename "$IMG")"
  echo
  echo "- Data (UTC): $STAMP"
  echo "- Host: $(uname -sm), $(qemu-system-aarch64 --version | head -1)"
  echo "- Máquina: virt + HVF, UEFI edk2, virtio-gpu-pci, virtio-net, USB teclado/tablet, 4 GiB, 4 vCPUs"
  echo "- SHA-256 da imagem: \`$(shasum -a 256 "$IMG" | cut -d' ' -f1)\`"
  echo "- Falhas: **$fails**"
  echo
  echo "## Etapas observadas pelo host"
  for s in "${steps[@]}"; do echo "- $s"; done
  echo
  echo "## Autoteste interno (serial)"
  echo '```'; clean | grep -a '^OSMTEST\|OSMTEST ' | sed 's/^.*OSMTEST/OSMTEST/'; echo '```'
  echo
  echo "## Capturas de tela"
  for f in "$SHOTS"/desktop-arm64-$STAMP-*.png; do [[ -e $f ]] && echo "![$(basename "$f")](screenshots/$(basename "$f"))"; done
} > "$REPORT"
clean > "$REPDIR/desktop-arm64-$STAMP-serial.log"
[[ ${OSM_KEEP_VM:-0} == 1 ]] || rm -f "$DISK" "${DISK%.*}-efivars.fd"
log "relatório: ${REPORT#$OSM_ROOT/}"
(( fails == 0 )) || die "teste de desktop ARM64: $fails falha(s)"
log "teste de desktop ARM64: OK"
exit 0
