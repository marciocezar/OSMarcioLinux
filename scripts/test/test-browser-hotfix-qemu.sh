#!/usr/bin/env bash
# Teste do hotfix do OSMarcio Browser dentro do OSMarcio Linux (QEMU/HVF):
#   1. sobe a imagem e faz o login gráfico real (test-desktop-qemu-arm64.sh, VM mantida ligada);
#   2. envia scripts/test/test-browser-hotfix.sh pela qemu-guest-agent e roda as fases
#      fechar → sobre-abrir → sobre-verificar → sobre-fechar → atualizacoes;
#   3. captura telas entre as fases (Sobre, barra de tarefas, Alt+Tab, menu de aplicativos);
#   4. grava reports/tests/browser-hotfix-arm64-<data>.md e desliga a VM.
# uso: test-browser-hotfix-qemu.sh <imagem.img>
source "$(dirname "$0")/../lib/common.sh"
IMG=${1:?imagem}
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
REPDIR="$OSM_ROOT/reports/tests"; SHOTS="$REPDIR/screenshots"; mkdir -p "$SHOTS"
REPORT="$REPDIR/browser-hotfix-arm64-$STAMP.md"
LOG="$OSM_ROOT/out/logs/browser-hotfix-$STAMP.log"; mkdir -p "$(dirname "$LOG")"

log "fase 0: boot + login gráfico (VM mantida ligada)"
out=$(OSM_KEEP_VM=1 "$OSM_ROOT/scripts/test/test-desktop-qemu-arm64.sh" "$IMG" 2>&1) || { echo "$out" | tail -5; die "desktop não subiu"; }
DREP=$(echo "$out" | sed 's/\x1b\[[0-9;]*m//g' | grep -o 'reports/tests/desktop-arm64-[0-9TZ]*\.md' | tail -1)
TDIR="$OSM_ROOT/out/test-desktop-$(basename "$DREP" .md | sed 's/desktop-arm64-//')"
[[ -S $TDIR/qga.sock ]] || die "socket do agente não encontrado em $TDIR"
QGA() { python3 "$OSM_ROOT/scripts/test/qga.py" "$TDIR/qga.sock" "$@"; }
MON() { python3 "$OSM_ROOT/scripts/test/qemu-monitor.py" "$TDIR/mon.sock" "$@"; }
shot() { MON screendump "$TDIR/$1.png"; cp "$TDIR/$1.png" "$SHOTS/browser-hotfix-arm64-$STAMP-$1.png"; echo "$1"; }
trap 'MON cmd quit >/dev/null 2>&1 || true' EXIT

QGA push "$OSM_ROOT/scripts/test/test-browser-hotfix.sh" /root/test-browser-hotfix.sh
fails=0; shots=()
phase() {
  log "fase: $1"
  local r=0
  QGA exec "bash /root/test-browser-hotfix.sh $1 2>&1" | tee -a "$LOG" || r=$?
  (( r == 0 )) || fails=$((fails + 1))
}
phase fechar
phase sobre-abrir
shots+=("$(shot 01-sobre-e-barra-de-tarefas)")
# Alt+Tab: segura Alt dentro da sessão (XTest) enquanto o host captura a tela.
QGA exec 'X="env DISPLAY=:0 XAUTHORITY=/home/osmarcio/.Xauthority"; ( $X xdotool keydown alt key Tab; sleep 4; $X xdotool keyup alt ) >/dev/null 2>&1 &' >/dev/null
sleep 1.5; shots+=("$(shot 02-alt-tab)"); sleep 3.5
QGA exec 'env DISPLAY=:0 XAUTHORITY=/home/osmarcio/.Xauthority xdotool mousemove 30 780 click 1 sleep 1.2 mousemove 60 697 sleep 1.5' >/dev/null
shots+=("$(shot 03-menu-internet)")
MON key esc; MON key esc; sleep 1
phase sobre-verificar
shots+=("$(shot 04-sobre-verificar-atualizacoes)")
phase sobre-fechar
phase atualizacoes

n_pass=$(grep -c '^PASS' "$LOG" || true); n_fail=$(grep -c '^FAIL' "$LOG" || true)
{
  echo "# Teste do hotfix do OSMarcio Browser — $(basename "$IMG")"
  echo
  echo "- Data (UTC): $STAMP · Host: $(uname -sm), $(qemu-system-aarch64 --version | head -1)"
  echo "- Sessão gráfica: login real no LightDM (ver \`$DREP\`)"
  echo "- SHA-256 da imagem: \`$(shasum -a 256 "$IMG" | cut -d' ' -f1)\`"
  echo "- Resultado: **$n_pass PASS, $n_fail FAIL**"
  echo
  echo '```'; grep -E '^(PASS|FAIL|LIMITACAO|RESULTADO)' "$LOG"; echo '```'
  echo
  echo "## Capturas de tela"
  for s in "${shots[@]}"; do echo "![$s](screenshots/browser-hotfix-arm64-$STAMP-$s.png)"; done
} > "$REPORT"
log "relatório: ${REPORT#$OSM_ROOT/}"
(( fails == 0 && n_fail == 0 )) || die "hotfix do browser: $n_fail falha(s)"
log "hotfix do browser: OK"
exit 0
