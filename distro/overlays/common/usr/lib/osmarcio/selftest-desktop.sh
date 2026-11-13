#!/bin/bash
# OSMarcio Linux — autoteste da SESSÃO GRÁFICA (fase 2 do selftest.sh).
#
# Só é iniciado pelo selftest.sh quando o hipervisor passa o modo "desktop"
# via fw_cfg. Espera o usuário entrar pelo LightDM (o host digita a senha
# pelo monitor do QEMU), então verifica, com evidência observável:
#   sessão XFCE (xfwm4, painel, xfdesktop) · resolução · OSMarcio Terminal
#   abre e executa comandos no bash · OSMarcio Browser abre, carrega HTTPS e
#   roda com sandbox. Cada resultado vira uma linha "OSMTEST" no serial.
set -u
MODE=${1:-desktop}
U=osmarcio
out() { echo "OSMTEST $*" > /dev/ttyAMA0 2>/dev/null || echo "OSMTEST $*" > /dev/console; }
pass() { out "PASS $*"; }
fail() { out "FAIL $*"; }

out "BEGIN-DESKTOP"
# 1. Espera a sessão gráfica do usuário (até 15 min: o login é externo).
for _ in $(seq 450); do
  pgrep -u "$U" -x xfce4-panel >/dev/null && break
  sleep 2
done
if ! pgrep -u "$U" -x xfce4-panel >/dev/null; then
  fail "desktop-session nenhuma sessão XFCE de '$U' em 15 min"
  out "END-DESKTOP"; [[ $MODE == *poweroff* ]] && systemctl poweroff; exit 0
fi
sleep 8   # deixa painel/desktop terminarem de desenhar

sess=$(pgrep -u "$U" -x xfce4-session | head -1)
envof() { tr '\0' '\n' < "/proc/$sess/environ" | sed -n "s/^$1=//p"; }
export DISPLAY=$(envof DISPLAY) XAUTHORITY=$(envof XAUTHORITY)
UENV=(DISPLAY="$DISPLAY" XAUTHORITY="$XAUTHORITY" XDG_RUNTIME_DIR="/run/user/$(id -u $U)"
      DBUS_SESSION_BUS_ADDRESS="$(envof DBUS_SESSION_BUS_ADDRESS)" HOME="/home/$U" LANG=pt_BR.UTF-8)
asuser() { runuser -u "$U" -- env "${UENV[@]}" "$@"; }

pass "desktop-session xfce4-session pid=$sess DISPLAY=$DISPLAY tipo=$(loginctl show-session "$(loginctl list-sessions --no-legend | awk -v u=$U '$3==u{print $1; exit}')" -p Type --value 2>/dev/null)"
for p in xfwm4 xfce4-panel xfdesktop; do
  pgrep -u "$U" -x "$p" >/dev/null && pass "desktop-$p em execução" || fail "desktop-$p não está em execução"
done
res=$(asuser xrandr 2>/dev/null | awk '/\*/{print $1; exit}')
[[ -n $res ]] && pass "desktop-resolution $res ($(asuser xrandr | awk '/ connected/{print $1; exit}'))" || fail "desktop-resolution xrandr sem modo ativo"

# 2. OSMarcio Terminal: abre janela e executa comando no shell.
# (o PID de "$!" seria o do runuser; a janela pertence ao processo do app)
asuser osmarcio-terminal >/tmp/osm-term.log 2>&1 &
win=""
for _ in $(seq 60); do
  tpid=$(pgrep -u "$U" -n -x osmarcio-termin)
  [[ -n $tpid ]] && win=$(asuser xdotool search --onlyvisible --pid "$tpid" 2>/dev/null | tail -1)
  [[ -n $win ]] && break; sleep 0.5
done
if [[ -n $win ]]; then
  pass "terminal-window janela X11 $win ($(asuser xdotool getwindowname "$win"))"
  rm -f /tmp/osmterm.txt
  asuser xdotool windowactivate --sync "$win" 2>/dev/null; sleep 1
  asuser xdotool type --delay 30 'echo OSMTERM-$((6*7)) $(id -un) $SHELL > /tmp/osmterm.txt'
  asuser xdotool key Return; sleep 2
  r=$(cat /tmp/osmterm.txt 2>/dev/null)
  [[ $r == "OSMTERM-42 $U /bin/bash" ]] && pass "terminal-shell bash executou comando digitado: '$r'" || fail "terminal-shell saída='$r'"
  asuser xdotool type --delay 30 'clear; osmarcio-info'; asuser xdotool key Return
else
  fail "terminal-window nenhuma janela (log: $(tail -1 /tmp/osm-term.log))"
fi

# 3. OSMarcio Browser: janela, HTTPS, sandbox.
BLOG=/tmp/osm-browser.log
asuser env RUST_LOG=info osmarcio-browser https://example.com >"$BLOG" 2>&1 &
waitlog() { for _ in $(seq $(( $2 * 2 ))); do grep -Eq "$1" "$BLOG" && return 0; sleep 0.5; done; return 1; }
waitlog 'osmb: janela criada' 60 && pass "browser-init $(grep -o 'OSMarcio Browser [^)]*)' "$BLOG" | head -1)" || fail "browser-init $(grep -v '^\s*$' "$BLOG" | tail -1 | cut -c1-110)"
bw=$(asuser xdotool search --name 'OSMarcio Browser' 2>/dev/null | head -1)
[[ -n $bw ]] && pass "browser-window janela X11 $bw" || fail "browser-window nenhuma janela 'OSMarcio Browser'"
waitlog 'load_end status=200 url=https://example.com' 90 && pass "browser-https example.com status 200" \
  || fail "browser-https $(grep -E 'load_error|load_end' "$BLOG" | tail -1 | cut -c1-110)"
ren=""
for p in $(pgrep -u "$U" -f /usr/lib/osmarcio-browser/osmarcio-browser); do
  [[ $(awk '/^Seccomp:/{print $2}' /proc/$p/status) == 2 ]] && { ren=$p; break; }
done
if [[ -n $ren ]]; then
  nsp=$(awk '/^NSpid:/{print NF-1}' "/proc/$ren/status")
  [[ $nsp -ge 2 ]] && pass "browser-sandbox renderer pid=$ren seccomp=2 pidns-aninhado" || fail "browser-sandbox pidns=$nsp"
else
  fail "browser-sandbox nenhum subprocesso com seccomp"
fi
pgrep -u "$U" -f -- '--no-sandbox' >/dev/null && fail "browser-no-sandbox há processo com --no-sandbox" || pass "browser-no-sandbox nenhum processo com --no-sandbox"
rss=0; for p in $(pgrep -u "$U" -f /usr/lib/osmarcio-browser/osmarcio-browser); do r=$(awk '/^VmRSS:/{print $2}' /proc/$p/status 2>/dev/null); rss=$((rss + ${r:-0})); done
out "INFO browser-rss-MiB=$((rss/1024)) mem-used-MiB=$(LC_ALL=C free -m | awk '/^Mem:/{print $3}')"

# Capturas de tela pelo host: 1) browser em primeiro plano (o host mede os
# pixels da área da página); 2) terminal em primeiro plano.
if [[ -n $bw ]]; then
  asuser xdotool windowactivate --sync "$bw" 2>/dev/null
  asuser xdotool windowmove "$bw" 0 0 windowsize "$bw" 1100 740 2>/dev/null
  sleep 3; out "INFO screenshot-browser"; sleep 10
fi
[[ -n $win ]] && asuser xdotool windowactivate "$win" 2>/dev/null
out "END-DESKTOP"
# O host captura a tela ao ver END-DESKTOP. Os apps rodam no cgroup desta
# unidade: sair agora os mataria antes da captura, então esperamos.
if [[ $MODE == *poweroff* ]]; then sleep 25; systemctl poweroff; else sleep 1800; fi
exit 0
