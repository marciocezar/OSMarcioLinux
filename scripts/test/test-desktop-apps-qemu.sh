#!/usr/bin/env bash
# Testes gráficos dos apps do desktop dentro do OSMarcio Linux (QEMU/HVF):
#   1. sobe a imagem e faz o login gráfico real (test-desktop-qemu-arm64.sh, VM mantida ligada);
#   2. envia scripts/test/test-desktop-apps.sh pela qemu-guest-agent e roda as fases;
#   3. captura telas entre as fases e mede a renderização do Browser maximizado;
#   4. grava reports/tests/desktop-apps-arm64-<data>.md e desliga a VM.
# uso: test-desktop-apps-qemu.sh <imagem.img>
source "$(dirname "$0")/../lib/common.sh"
IMG=${1:?imagem}
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
REPDIR="$OSM_ROOT/reports/tests"; SHOTS="$REPDIR/screenshots"; mkdir -p "$SHOTS"
REPORT="$REPDIR/desktop-apps-arm64-$STAMP.md"
LOG="$OSM_ROOT/out/logs/desktop-apps-$STAMP.log"; mkdir -p "$(dirname "$LOG")"

log "fase 0: boot + login gráfico (VM mantida ligada)"
out=$(OSM_KEEP_VM=1 "$OSM_ROOT/scripts/test/test-desktop-qemu-arm64.sh" "$IMG" 2>&1) || { echo "$out" | tail -5; die "desktop não subiu"; }
DREP=$(echo "$out" | sed 's/\x1b\[[0-9;]*m//g' | grep -o 'reports/tests/desktop-arm64-[0-9TZ]*\.md' | tail -1)
TDIR="$OSM_ROOT/out/test-desktop-$(basename "$DREP" .md | sed 's/desktop-arm64-//')"
QGA() { python3 "$OSM_ROOT/scripts/test/qga.py" "$TDIR/qga.sock" "$@"; }
MON() { python3 "$OSM_ROOT/scripts/test/qemu-monitor.py" "$TDIR/mon.sock" "$@"; }
shot() { MON screendump "$TDIR/$1.png"; cp "$TDIR/$1.png" "$SHOTS/desktop-apps-arm64-$STAMP-$1.png"; shots+=("$1"); }
trap 'MON cmd quit >/dev/null 2>&1 || true' EXIT
QGA push "$OSM_ROOT/scripts/test/test-desktop-apps.sh" /root/test-desktop-apps.sh
fails=0; shots=()
phase() { log "fase: $1"; local r=0; QGA exec "bash /root/test-desktop-apps.sh $1 2>&1" | tee -a "$LOG" || r=$?; (( r == 0 )) || fails=$((fails + 1)); }

phase terminal-1;  shot 01-terminal-1-aba
phase terminal-5;  shot 02-terminal-5-abas
phase terminal-10; shot 03-terminal-10-abas
phase terminal-fim
phase browser-max; sleep 1; shot 04-browser-maximizado
# Barra de ferramentas (★ ✕ ☰) e barra de rolagem da página junto à borda direita
# só existem ali se a interface e a página acompanharam a nova largura.
st1=$(python3 "$OSM_ROOT/scripts/test/qemu-monitor.py" - pngstat "$TDIR/04-browser-maximizado.png" 1180 55 1279 95)
st2=$(python3 "$OSM_ROOT/scripts/test/qemu-monitor.py" - pngstat "$TDIR/04-browser-maximizado.png" 1262 100 1279 720)
c1=$(echo "$st1" | sed -E 's/.*colors=([0-9]+).*/\1/'); c2=$(echo "$st2" | sed -E 's/.*colors=([0-9]+).*/\1/')
if (( c1 >= 5 && c2 >= 3 )); then echo "PASS browser-render barra e página desenhadas até a borda direita (barra: $st1 · rolagem: $st2)" | tee -a "$LOG"
else echo "FAIL browser-render borda direita sem interface/página (barra: $st1 · rolagem: $st2)" | tee -a "$LOG"; fails=$((fails+1)); fi
phase browser-fim
phase files-a; shot 05-files-lista
phase files-cancelar
phase files-copia-inicio; sleep 0.3; shot 06-files-progresso
phase files-copia-fim
phase files-b; shot 07-files-propriedades
phase files-icones; shot 08-files-icones
phase files-fim
phase minas-vitoria; shot 09-minas-vitoria
phase minas-derrota; shot 10-minas-derrota
phase minas-recorde; shot 11-minas-vitoria-facil
phase minas-fim

# Logs dos apps (diagnóstico) ao lado do log do teste.
for f in files term br ms; do QGA pull "/tmp/osm-apps/$f.log" "$OSM_ROOT/out/logs/desktop-apps-$STAMP-$f.log" 2>/dev/null || true; done
n_pass=$(grep -c '^PASS' "$LOG" || true); n_fail=$(grep -c '^FAIL' "$LOG" || true)
{
  echo "# Testes gráficos dos apps — $(basename "$IMG")"
  echo
  echo "- Data (UTC): $STAMP · Host: $(uname -sm), $(qemu-system-aarch64 --version | head -1)"
  echo "- Sessão gráfica: login real no LightDM (ver \`$DREP\`) · resolução 1280x800 · XFCE/xfwm4"
  echo "- SHA-256 da imagem: \`$(shasum -a 256 "$IMG" | cut -d' ' -f1)\`"
  echo "- Resultado: **$n_pass PASS, $n_fail FAIL**"
  echo
  echo '```'; grep -E '^(PASS|FAIL|RESULTADO)' "$LOG"; echo '```'
  echo
  echo "## Capturas de tela"
  for s in "${shots[@]}"; do echo "![$s](screenshots/desktop-apps-arm64-$STAMP-$s.png)"; done
} > "$REPORT"
log "relatório: ${REPORT#$OSM_ROOT/}"
(( fails == 0 && n_fail == 0 )) || die "apps do desktop: $n_fail falha(s)"
log "apps do desktop: OK"
exit 0
