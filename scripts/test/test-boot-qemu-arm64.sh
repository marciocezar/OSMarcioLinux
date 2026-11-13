#!/usr/bin/env bash
# Teste de integração real: inicializa a imagem ARM64 DUAS vezes no QEMU
# (UEFI -> GRUB -> kernel Linux -> systemd) e coleta as linhas OSMTEST
# emitidas pelo autoteste interno (distro/overlays/.../selftest.sh).
#
# A segunda inicialização usa o mesmo disco e prova persistência
# (boot-count deve ir de 1 para 2).
#
# uso: test-boot-qemu-arm64.sh <imagem.img>
source "$(dirname "$0")/../lib/common.sh"
need qemu-img

IMG=${1:?imagem}
[[ -f $IMG ]] || die "imagem não encontrada: $IMG"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
TDIR="$OSM_ROOT/out/test-$STAMP"; mkdir -p "$TDIR"
REPDIR="$OSM_ROOT/reports/tests"; mkdir -p "$REPDIR"
REPORT="$REPDIR/boot-arm64-$STAMP.md"

# Overlay descartável: a imagem testada nunca é alterada.
DISK="$TDIR/disk.qcow2"
qemu-img create -q -f qcow2 -F raw -b "$(cd "$(dirname "$IMG")" && pwd)/$(basename "$IMG")" "$DISK" 20G

TIMEOUT=${TIMEOUT:-900}
boot() { # boot <n>
  local log="$TDIR/serial-boot$1.log" t0=$SECONDS
  log "boot $1 (timeout ${TIMEOUT}s) — serial em $log"
  "$OSM_ROOT/scripts/run/qemu-arm64.sh" "$IMG" --disk "$DISK" --gpu-headless \
      --selftest poweroff --serial-log "$log" --mem 2048 --cpus 2 &
  local pid=$!
  while kill -0 $pid 2>/dev/null; do
    if (( SECONDS - t0 > TIMEOUT )); then
      warn "timeout no boot $1 — encerrando QEMU"; kill $pid; wait $pid 2>/dev/null; break
    fi
    sleep 2
  done
  wait $pid 2>/dev/null || true
  echo $(( SECONDS - t0 ))
}

d1=$(boot 1); d2=$(boot 2)

L1="$TDIR/serial-boot1.log"; L2="$TDIR/serial-boot2.log"
results() { tr -d '\r' < "$1" | grep -a '^OSMTEST' || true; }
fails=0
grep -aq 'OSMTEST END' "$L1" || { warn "boot 1 não concluiu o autoteste"; fails=$((fails+1)); }
grep -aq 'OSMTEST END' "$L2" || { warn "boot 2 não concluiu o autoteste"; fails=$((fails+1)); }
n_fail=$(cat "$L1" "$L2" | grep -ac '^OSMTEST FAIL' || true)
fails=$((fails + n_fail))
grep -aq 'boot-count=1' "$L1" && grep -aq 'boot-count=2' "$L2" && persist=PASS || { persist=FAIL; fails=$((fails+1)); }

{
  echo "# Teste de boot ARM64 — $(basename "$IMG")"
  echo
  echo "- Data (UTC): $STAMP"
  echo "- Host: $(uname -sm), $(qemu-system-aarch64 --version | head -1)"
  echo "- SHA-256 da imagem: \`$(shasum -a 256 "$IMG" | cut -d' ' -f1)\`"
  echo "- Duração: boot 1 = ${d1}s, boot 2 = ${d2}s (inclui apt update/install)"
  echo "- Persistência entre boots: **$persist**"
  echo "- Falhas: **$fails**"
  echo
  echo "## Boot 1"; echo '```'; results "$L1"; echo '```'
  echo "## Boot 2"; echo '```'; results "$L2"; echo '```'
  echo "## Trechos do console (boot 1)"; echo '```'
  tr -d '\r' < "$L1" | grep -aE 'Linux version|EFI stub|Welcome to|osmarcio-firstboot|login:' | sed 's/\x1b\[[0-9;]*m//g' | head -20
  echo '```'
} > "$REPORT"

cp "$L1" "$L2" "$REPDIR/" 2>/dev/null && mv "$REPDIR/serial-boot1.log" "$REPDIR/boot-arm64-$STAMP-serial1.log" && mv "$REPDIR/serial-boot2.log" "$REPDIR/boot-arm64-$STAMP-serial2.log"
rm -f "$DISK" "${DISK%.*}-efivars.fd"

log "relatório: ${REPORT#$OSM_ROOT/}"

# Registra o resultado no manifesto do artefato (se houver).
MANIFEST="$IMG.manifest.md"
if [[ -f $MANIFEST ]]; then
  res=$([[ $fails -eq 0 ]] && echo "APROVADO" || echo "REPROVADO ($fails falhas)")
  python3 - "$MANIFEST" "$STAMP" "$res" "${REPORT#$OSM_ROOT/}" <<'PY'
import sys
p, stamp, res, rep = sys.argv[1:]
s = open(p).read().replace("Ainda não testado. Rode `make test-utm` (ou o alvo de teste da plataforma);\no relatório vai para `reports/tests/`.\n", "")
s = s.replace("## Resultado dos testes\n", f"## Resultado dos testes\n- {stamp}: boot QEMU/HVF (UEFI, 2 boots) — **{res}** — `{rep}`\n", 1)
open(p, "w").write(s)
PY
fi
if (( fails > 0 )); then die "teste de boot ARM64: $fails falha(s)"; fi
log "teste de boot ARM64: OK"
