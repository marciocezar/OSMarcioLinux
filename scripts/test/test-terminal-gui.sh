#!/usr/bin/env bash
# Teste de interface REAL do OSMarcio Terminal (roda DENTRO do container):
# sobe um servidor X virtual (Xvfb), abre o terminal e o dirige com xdotool,
# verificando efeitos observáveis (arquivos criados pelo shell, logs de abas).
source "$(dirname "$0")/../lib/common.sh"
need Xvfb; need xdotool

BIN=${CARGO_TARGET_DIR:-$OSM_ROOT/target}/debug/osmarcio-terminal
[[ -x $BIN ]] || cargo build -p osmarcio-terminal
export HOME=$(mktemp -d) SHELL=/bin/bash DISPLAY=:99 GSK_RENDERER=cairo NO_AT_BRIDGE=1 RUST_LOG=info
LOG=$HOME/terminal.log
Xvfb :99 -screen 0 1280x800x24 >/dev/null 2>&1 & XPID=$!
trap 'kill $TPID $XPID 2>/dev/null; true' EXIT
sleep 1

dbus-run-session -- "$BIN" >"$LOG" 2>&1 & TPID=$!
WID=$(xdotool search --sync --onlyvisible --name "OSMarcio Terminal" | head -1)
xdotool windowactivate --sync "$WID" 2>/dev/null || xdotool windowfocus --sync "$WID"
sleep 1.5

PASS=0; FAIL=0
ok()   { echo "PASS $*"; PASS=$((PASS+1)); }
bad()  { echo "FAIL $*"; FAIL=$((FAIL+1)); }
run()  { xdotool type --delay 15 "$1"; xdotool key Return; }
waitfile() { for _ in $(seq 50); do [[ -s $1 ]] && return 0; sleep 0.2; done; return 1; }
lastabas() { grep -o 'abas=[0-9]* botoes_fechar=[a-z]*' "$LOG" | tail -1; }

[[ $(lastabas) == "abas=1 botoes_fechar=false" ]] && ok "1 aba, sem botão X" || bad "estado inicial: $(lastabas)"

run 'echo osm-pipe | tr a-z A-Z > ~/pipe.txt'
waitfile "$HOME/pipe.txt" && [[ $(cat "$HOME/pipe.txt") == OSM-PIPE ]] && ok "bash: pipe + redirecionamento" || bad "pipe"

run 'echo marcador-historico >/dev/null; history > ~/hist.txt'
waitfile "$HOME/hist.txt" && grep -q marcador-historico "$HOME/hist.txt" && ok "histórico do bash" || bad "histórico"

run 'sleep 300'; sleep 0.5; xdotool key ctrl+c; sleep 0.3
run 'echo depois-ctrl-c > ~/ctrlc.txt'
waitfile "$HOME/ctrlc.txt" && ok "Ctrl+C interrompe processo" || bad "Ctrl+C"

xdotool key ctrl+w; sleep 0.8
kill -0 $TPID 2>/dev/null && [[ $(lastabas) == "abas=1 botoes_fechar=false" ]] && ok "Ctrl+W não fecha a última aba" || bad "Ctrl+W na última aba"

xdotool key ctrl+t; sleep 1.5
[[ $(lastabas) == "abas=2 botoes_fechar=true" ]] && ok "Ctrl+T abre 2ª aba e botões X reaparecem" || bad "Ctrl+T: $(lastabas)"
run 'echo $$ > ~/tab2.pid'
waitfile "$HOME/tab2.pid" && ok "2ª aba tem sessão própria (pid $(cat "$HOME/tab2.pid"))" || bad "sessão aba 2"

xdotool key ctrl+Tab; sleep 0.5
run 'echo $$ > ~/tab1.pid'
waitfile "$HOME/tab1.pid" && [[ $(cat "$HOME/tab1.pid") != $(cat "$HOME/tab2.pid") ]] && ok "Ctrl+Tab alterna para a aba 1 (sessão independente)" || bad "Ctrl+Tab"

xdotool key ctrl+w; sleep 1
[[ $(lastabas) == "abas=1 botoes_fechar=false" ]] && ok "Ctrl+W fecha aba quando há 2; X some" || bad "Ctrl+W com 2 abas: $(lastabas)"

run 'exit'
for _ in $(seq 25); do kill -0 $TPID 2>/dev/null || break; sleep 0.2; done
kill -0 $TPID 2>/dev/null && bad "exit na última aba deveria encerrar a janela" || ok "exit na última aba encerra o app"

echo "RESULTADO: $PASS passaram, $FAIL falharam"
[[ $FAIL -eq 0 ]] || { echo "--- log do app ---"; cat "$LOG"; exit 1; }
