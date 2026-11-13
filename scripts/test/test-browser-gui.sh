#!/usr/bin/env bash
# Teste REAL do OSMarcio Browser (Rust + CEF) — roda DENTRO do container
# (ou dentro do OSMarcio Linux), sob X11 (Xvfb ou a sessão existente).
#
# Verifica por evidência observável (logs "osmb:" do app e /proc):
#   CEF inicializa · HTTPS carrega · JavaScript/DOM/localStorage executam ·
#   sandbox ativo nos renderers (seccomp + namespaces) · barra de endereços ·
#   abas (Ctrl+T / Ctrl+W) · erro de DNS · certificado inválido rejeitado ·
#   download · memória · encerramento sem processos órfãos.
#
# uso: test-browser-gui.sh <diretório do bundle com osmarcio-browser + CEF>
# Requer: xdotool, curl; Xvfb se não houver $DISPLAY. Não pode rodar como root
# (o próprio script se reexecuta como o usuário "tester" quando é root).
set -uo pipefail

BUNDLE=${1:?bundle}
BIN="$BUNDLE/osmarcio-browser"
[[ -x $BIN ]] || { echo "binário não encontrado: $BIN"; exit 2; }
if [[ $(id -u) == 0 ]]; then
  exec runuser -u tester -- env PATH="$PATH" "$0" "$@"
fi

export HOME=$(mktemp -d) RUST_LOG=info OSMARCIO_BROWSER_SELFTEST=1 NO_AT_BRIDGE=1
LOG=$HOME/browser.log
if [[ -z ${DISPLAY:-} ]]; then
  export DISPLAY=:98
  Xvfb :98 -screen 0 1280x800x24 >/dev/null 2>&1 & XPID=$!
  sleep 1
fi

PASS=0; FAIL=0
ok()  { echo "PASS $*"; PASS=$((PASS+1)); }
bad() { echo "FAIL $*"; FAIL=$((FAIL+1)); }
waitlog() { # waitlog <regex> <segundos>
  local t=$(( ${2:-30} * 5 ))
  for _ in $(seq "$t"); do grep -Eq "$1" "$LOG" && return 0; sleep 0.2; done; return 1
}
procs() { pgrep -u "$(id -u)" -f "$BUNDLE/osmarcio-browser" ; }
WIN=""
focus() { # dá foco real à janela do navegador (sem gerenciador de janelas no Xvfb)
  [[ -n $WIN ]] || WIN=$(xdotool search --name "OSMarcio Browser" 2>/dev/null | head -1)
  [[ -n $WIN ]] || return 0
  xdotool windowfocus --sync "$WIN" 2>/dev/null
  eval "$(xdotool getwindowgeometry --shell "$WIN")"
  xdotool mousemove --window "$WIN" $((WIDTH/2)) $((HEIGHT/2)) click 1
}
key() { focus; xdotool key --clearmodifiers "$@"; }
go() { key ctrl+l; sleep 0.3; xdotool type --delay 10 "$1"; key Return; }

t0=$(date +%s.%N)
"$BIN" https://example.com >"$LOG" 2>&1 & BPID=$!
trap 'kill $BPID ${XPID:-} 2>/dev/null; true' EXIT

waitlog 'osmb: janela criada' 30 && ok "CEF inicializou e a janela Rust foi criada ($(grep -o 'CEF [^)]*' "$LOG" | head -1))" || bad "CEF não inicializou"
if waitlog 'load_end status=200 url=https://example.com' 60; then
  t1=$(date +%s.%N)
  ok "HTTPS: example.com carregou (status 200) em $(printf '%.1f' "$(echo "$t1 - $t0" | bc)") s"
else
  bad "HTTPS example.com"
fi
waitlog 'title=OSMB-JS:42:dom=[0-9]+:ls=ok:ua=chromium' 20 \
  && ok "JavaScript + DOM + localStorage: $(grep -o 'OSMB-JS:[^ ]*' "$LOG" | head -1)" || bad "JavaScript"

# Sandbox: renderers devem estar sob seccomp-bpf (Seccomp: 2) e em outro PID
# namespace. No Linux os renderers são filhos do "zygote" e herdam a linha de
# comando dele, então são identificados pelo estado em /proc.
sleep 1
ren=""
for p in $(procs); do
  if [[ $(awk '/^Seccomp:/{print $2}' /proc/$p/status) == 2 ]]; then ren=$p; break; fi
done
if [[ -n $ren ]]; then
  sec=$(awk '/^Seccomp:/{print $2}' "/proc/$ren/status")
  nsp=$(awk '/^NSpid:/{print NF-1}' "/proc/$ren/status")
  [[ $sec == 2 && $nsp -ge 2 ]] && ok "sandbox ativo no renderer (pid $ren): seccomp=$sec, PID namespace aninhado" \
    || bad "sandbox: seccomp=$sec níveis_pidns=$nsp"
  grep -q -- '--no-sandbox' "/proc/$ren/cmdline" && bad "renderer com --no-sandbox" || ok "nenhum processo com --no-sandbox"
else
  bad "nenhum processo renderer encontrado"
fi
[[ $(id -u) != 0 ]] && ok "executando como usuário sem privilégios ($(id -un))"

# Barra de endereços + navegação.
go "https://www.debian.org/"
waitlog 'load_end status=200 url=https://www.debian.org/' 40 && ok "navegação pela barra de endereços (debian.org)" || bad "navegação"

# Abas.
key ctrl+t
waitlog 'aba aberta id=2 total=2' 15 && ok "Ctrl+T abriu 2ª aba" || bad "Ctrl+T"
sleep 1
key ctrl+w
waitlog 'restantes=1' 15 && ok "Ctrl+W fechou a aba (restam 1)" || bad "Ctrl+W"

# Erro de DNS -> página de erro própria.
go "https://nao-existe.osmarcio.invalid/"
waitlog 'load_error code=-105' 30 && ok "erro DNS tratado (ERR_NAME_NOT_RESOLVED -105)" || bad "erro DNS"

# Certificado inválido -> rejeitado, sem opção de ignorar.
go "https://expired.badssl.com/"
if waitlog 'certificado rejeitado|load_error code=-20[0-9]' 30; then
  ok "certificado expirado rejeitado ($(grep -Eo 'certificado rejeitado code=-?[0-9]+|load_error code=-20[0-9]' "$LOG" | head -1))"
else
  bad "certificado inválido não foi rejeitado"
fi
grep -q 'load_end status=200 url=https://expired.badssl.com' "$LOG" && bad "página com certificado inválido foi exibida"

# Download (arquivo .deb pequeno do repositório Debian).
DEB=$(curl -fsS https://deb.debian.org/debian/pool/main/h/hello/ | grep -Eo 'hello_[^"<>]*_arm64\.deb' | head -1)
if [[ -n $DEB ]]; then
  go "https://deb.debian.org/debian/pool/main/h/hello/$DEB"
  if waitlog 'download_complete path=' 60; then
    f=$(grep -o 'download_complete path=.*' "$LOG" | head -1 | cut -d= -f2-)
    [[ -s $f ]] && ok "download concluído: $(basename "$f") ($(stat -c %s "$f") bytes)" || bad "download vazio"
  else
    bad "download"
  fi
else
  bad "não foi possível descobrir um .deb de teste (curl)"
fi

# Memória total (RSS) de todos os processos do navegador.
rss=0; n=0
for p in $(procs); do r=$(awk '/^VmRSS:/{print $2}' "/proc/$p/status" 2>/dev/null); rss=$((rss + ${r:-0})); n=$((n+1)); done
ok "memória: $n processos, RSS somado $((rss/1024)) MiB"

# Encerramento: Ctrl+Q fecha tudo, sem órfãos.
key ctrl+q
for _ in $(seq 50); do kill -0 $BPID 2>/dev/null || break; sleep 0.2; done
sleep 1
left=$(procs | wc -l)
kill -0 $BPID 2>/dev/null && bad "processo principal não terminou" || ok "Ctrl+Q encerrou o navegador"
[[ $left -eq 0 ]] && ok "nenhum subprocesso Chromium órfão" || bad "$left subprocesso(s) restantes"

echo "RESULTADO: $PASS passaram, $FAIL falharam"
[[ $FAIL -eq 0 ]] || { echo "--- log ---"; grep -v '^\s*$' "$LOG" | tail -40; exit 1; }
