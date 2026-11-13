#!/bin/bash
# Teste do hotfix do OSMarcio Browser — roda DENTRO do OSMarcio Linux (root),
# com a sessão XFCE do usuário osmarcio aberta em :0 (ver
# scripts/test/test-desktop-qemu-arm64.sh). Enviado e executado pelo host via
# qemu-guest-agent (scripts/test/test-browser-hotfix-qemu.sh).
#
# uso: test-browser-hotfix.sh <fase>   fases: fechar | sobre-abrir | sobre-verificar | sobre-fechar | atualizacoes
# (o host captura telas entre as fases). Estado entre fases fica em /tmp/osmb-test.
#
# Cada cenário imprime "PASS|FAIL <nome> <detalhe>". Nada é simulado: as
# janelas são fechadas por cliques/teclas reais (XTest) e o encerramento é
# verificado pela ausência de processos do navegador do usuário.
set -u
PHASE=${1:?fase}
U=osmarcio
BIN=$(readlink -f "$(command -v osmarcio-browser)")
X=(env DISPLAY=:0 XAUTHORITY=/home/$U/.Xauthority)
PASS=0; FAIL=0
ok()  { echo "PASS $*"; PASS=$((PASS+1)); }
bad() { echo "FAIL $*"; FAIL=$((FAIL+1)); }
procs() { pgrep -u "$U" -f "$BIN" | wc -l; }
xdo() { "${X[@]}" xdotool "$@"; }

launch() { # launch <log> [args...] — abre o browser como o usuário da sessão
  local log=$1; shift
  runuser -u "$U" -- env DISPLAY=:0 XAUTHORITY=/home/$U/.Xauthority HOME=/home/$U \
    XDG_RUNTIME_DIR=/run/user/1000 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus \
    LANG=pt_BR.UTF-8 RUST_LOG=info osmarcio-browser "$@" >"$log" 2>&1 &
  for _ in $(seq 60); do grep -q 'osmb: janela criada' "$log" 2>/dev/null && break; sleep 0.5; done
  sleep 1.5
  W=$(xdo search --name 'OSMarcio Browser' | tail -1)
  # posição fixa: o botão X da decoração do xfwm4 fica em (1086,13)
  xdo windowactivate --sync "$W" windowmove "$W" 0 0 windowsize "$W" 1100 740 2>/dev/null
  sleep 1
}
waitlog() { for _ in $(seq $(( $3 * 2 ))); do grep -Eq "$2" "$1" && return 0; sleep 0.5; done; return 1; }
gone() { # gone <log> <cenário> — espera o processo terminar e confere órfãos
  local t0=$SECONDS
  for _ in $(seq 40); do [[ $(procs) -eq 0 ]] && break; sleep 0.5; done
  local n; n=$(procs)
  if [[ $n -eq 0 ]] && grep -q 'osmb: encerrado' "$1"; then
    ok "$2 janela fechada, processo encerrado normalmente em $((SECONDS - t0))s, 0 subprocessos"
  else
    bad "$2 restam $n processos; log: $(grep -E 'fechar|destru|encerr' "$1" | tail -2 | tr '\n' ' ' | cut -c1-160)"
    pkill -u "$U" -f "$BIN"; sleep 2
  fi
}

W=$(cat /tmp/osmb-test/W 2>/dev/null); A=$(cat /tmp/osmb-test/A 2>/dev/null)
mkdir -p /tmp/osmb-test
# O conffile do pacote é alterado pelos testes de atualização: guarda o
# original uma única vez e o devolve ao fim de cada fase que mexe nele.
CONF=/etc/osmarcio/browser-updates.toml
[[ -f /tmp/osmb-test/conf.orig || ! -f $CONF ]] || cp -p "$CONF" /tmp/osmb-test/conf.orig
restore_conf() { [[ -f /tmp/osmb-test/conf.orig ]] && cp -p /tmp/osmb-test/conf.orig "$CONF"; }
case $PHASE in
fechar)
pkill -u "$U" -f "$BIN"; sleep 2

# Servidor local lento (página "carregando" e download em andamento).
cat > /tmp/slow.py <<'PY'
import http.server, socketserver, time
class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def do_GET(self):
        if self.path.startswith("/file"):
            self.send_response(200)
            self.send_header("Content-Type", "application/octet-stream")
            self.send_header("Content-Disposition", "attachment; filename=grande.bin")
            self.send_header("Content-Length", str(200 * 1024 * 1024))
            self.end_headers()
            try:
                for _ in range(2000):
                    self.wfile.write(b"\0" * 65536); self.wfile.flush(); time.sleep(0.2)
            except Exception:
                pass
        else:
            time.sleep(120)
socketserver.ThreadingTCPServer.allow_reuse_address = True
socketserver.ThreadingTCPServer(("127.0.0.1", 8765), H).serve_forever()
PY
python3 /tmp/slow.py & SLOW=$!
sleep 1

# 1. Uma aba + clique no X da janela.
launch /tmp/t1.log https://example.com
waitlog /tmp/t1.log 'load_end status=200' 30
xdo mousemove 1086 13 click 1
gone /tmp/t1.log "fechar-x-1-aba"

# 2. Três abas + Alt+F4.
launch /tmp/t2.log https://example.com
xdo key ctrl+t; sleep 1; xdo key ctrl+t; sleep 1
waitlog /tmp/t2.log 'aba aberta id=3 total=3' 10 && ok "tres-abas abertas" || bad "tres-abas não abriram"
xdo windowactivate --sync "$W" key alt+F4
gone /tmp/t2.log "fechar-altf4-3-abas"

# 3. Página carregando + Ctrl+Q.
launch /tmp/t3.log http://127.0.0.1:8765/lento
grep -q 'load_end' /tmp/t3.log && bad "carregando a página não estava carregando" || ok "carregando página em carregamento (servidor lento)"
xdo mousemove 550 400 click 1 key ctrl+q
gone /tmp/t3.log "fechar-ctrlq-carregando"

# 4. Download em andamento + X.
launch /tmp/t4.log https://example.com
xdo key ctrl+l; sleep 0.3; xdo type --delay 15 'http://127.0.0.1:8765/file'; xdo key Return
waitlog /tmp/t4.log 'download_start' 20 && ok "download iniciado ($(grep -o 'download_start path=[^ ]*' /tmp/t4.log | head -1))" || bad "download não iniciou"
sleep 2
xdo mousemove 1086 13 click 1
gone /tmp/t4.log "fechar-x-download-ativo"
rm -f /home/$U/Downloads/grande.bin* "/home/$U/Downloads/(1) grande.bin"*

# 5. Menu ☰ → "Fechar janela" (último item: seta para cima).
launch /tmp/t5.log https://example.com
xdo mousemove 1082 77 click 1; sleep 1.5
waitlog /tmp/t5.log 'osmb: menu aberto' 5 && ok "menu aberto pelo botão ☰" || bad "menu não abriu"
xdo key Up; sleep 0.3; xdo key Return
gone /tmp/t5.log "fechar-menu"

# 6. Ctrl+W com várias abas fecha só a aba; na última aba fecha a janela.
launch /tmp/t6.log https://example.com
xdo key ctrl+t; sleep 1.5
xdo key ctrl+w
waitlog /tmp/t6.log 'restantes=1' 10 && [[ $(procs) -gt 0 ]] && ok "ctrl-w fechou só a aba (janela continua aberta)" || bad "ctrl-w"
sleep 1; xdo key ctrl+w
gone /tmp/t6.log "fechar-ctrlw-ultima-aba"

kill $SLOW 2>/dev/null
;;
sobre-abrir)
pkill -u "$U" -f "$BIN"; sleep 2
# 7. Reabrir depois de fechar.
launch /tmp/t7.log https://example.com
waitlog /tmp/t7.log 'load_end status=200' 30 && ok "reabrir navegador abriu e carregou de novo" || bad "reabrir"

# 8. Ícone e identificação da janela.
cls=$("${X[@]}" xprop -id "$W" WM_CLASS 2>/dev/null)
# WM_CLASS: limitação conhecida (docs/browser/OSMarcioBrowser_Icons.md) — informado, não contado.
echo "$cls" | grep -q '"osmarcio-browser"' && ok "wm-class $cls" || echo "LIMITACAO wm-class ausente ($cls)"
icons=$("${X[@]}" xprop -id "$W" _NET_WM_ICON | grep -o 'Icon ([0-9]* x [0-9]*)' | tr '\n' ' ')
[[ $icons == *"16 x 16"* && $icons == *"128 x 128"* ]] && ok "icone-janela _NET_WM_ICON: $icons" || bad "icone-janela $icons"

# 9. Sobre (F1): abre, versões reais, fechar só o Sobre.
xdo windowactivate --sync "$W"; xdo key F1
if waitlog /tmp/t7.log 'osmb: sobre aberto' 15; then
  ok "sobre-abrir $(grep -o 'sobre aberto .*' /tmp/t7.log | head -1 | cut -c14-)"
else
  bad "sobre não abriu"
fi
sleep 2
A=$(xdo search --name 'Sobre o OSMarcio Browser' | tail -1)
[[ -n $A ]] && ok "sobre-janela X11 $A" || bad "sobre-janela não encontrada"
v=$(runuser -u "$U" -- osmarcio-browser --version)
grep -q "chromium=$(echo "$v" | awk '/Chromium Engine/{print $3}')" /tmp/t7.log && ok "sobre-chromium igual ao --version ($(echo "$v" | awk '/Chromium Engine/{print $3}'))" || bad "sobre-chromium"
grep -q 'runtime_confere=true' /tmp/t7.log && ok "sobre-runtime libcef.so carregado == versão compilada" || bad "sobre-runtime difere"
echo "$W" > /tmp/osmb-test/W; echo "$A" > /tmp/osmb-test/A
;;
sobre-verificar)
# Botões do Sobre (janela em 0,0): Verificar (74,570), Copiar (224,570).
echo 'manifest_url = ""' > /etc/osmarcio/browser-updates.toml
xdo windowactivate --sync "$A" mousemove 74 570 click 1
waitlog /tmp/t7.log 'osmb: atualizacoes:' 30 && ok "sobre-verificar $(grep -o 'atualizacoes: .*' /tmp/t7.log | tail -1 | cut -c15-150)" || bad "sobre-verificar sem resposta"
grep -q 'atualizacoes: Fonte de atualização não configurada' /tmp/t7.log && ok "sobre-verificar estado 'fonte não configurada' (padrão do pacote)" || bad "sobre-verificar estado inesperado"
xdo mousemove 224 570 click 1; sleep 1
xdo windowactivate --sync "$W" key ctrl+l; sleep 0.3; xdo key ctrl+a ctrl+v Return; sleep 1
grep -q 'barra de endereços -> .*Chromium Engine' /tmp/t7.log && ok "sobre-copiar texto copiado contém as versões (colado na barra de endereços)" || bad "sobre-copiar"
xdo windowactivate --sync "$A"; xdo mousemove 74 570 click 1; sleep 3
restore_conf
;;
sobre-fechar)
xdo windowactivate --sync "$A" mousemove 328 570 click 1
waitlog /tmp/t7.log 'osmb: sobre fechado' 10 && [[ $(procs) -gt 0 ]] && ok "sobre-fechar botão Fechar fechou só o Sobre; navegador segue aberto" || bad "sobre-fechar"

# 10. Fechar a janela principal com o Sobre aberto fecha os dois.
xdo windowactivate --sync "$W"; xdo key F1; waitlog /tmp/t7.log 'sobre aberto.*\n?' 10; sleep 2
xdo windowactivate --sync "$W"; xdo mousemove 1086 13 click 1
gone /tmp/t7.log "fechar-x-com-sobre-aberto"


;;
atualizacoes)
CUR=$(runuser -u "$U" -- osmarcio-browser --version | awk '/Versão do Browser/{print $4}')
ARCHD=$(dpkg --print-architecture)
OTHER=$([[ $ARCHD == arm64 ]] && echo amd64 || echo arm64)
SHA=$(printf '%064d' 0)
man() { printf '[[release]]\nversion = "%s"\narch = "%s"\nchromium = "157.0.0.0"\nhomologated = %s\nsha256 = "%s"\n' "$@" > /tmp/osmb-test/m.toml; }
chk() { # chk <nome> <regex esperado> <manifest_url>
  echo "manifest_url = \"$3\"" > "$CONF"
  local r; r=$(runuser -u "$U" -- osmarcio-browser --check-updates 2>&1)
  [[ $r =~ $2 ]] && ok "atualizacao-$1: $r" || bad "atualizacao-$1: esperado /$2/, obtido: $r"
}
chk fonte-ausente 'não configurada' ''
man "$CUR" "$ARCHD" true "$SHA";        chk ja-atualizado 'está atualizado' file:///tmp/osmb-test/m.toml
man 9.9.9 "$ARCHD" true "$SHA";         chk nova-homologada 'homologada 9\.9\.9.*não há repositório APT' file:///tmp/osmb-test/m.toml
man 9.9.9 "$ARCHD" false "$SHA";        chk nao-homologada 'ainda não homologada' file:///tmp/osmb-test/m.toml
man 9.9.9 "$OTHER" true "$SHA";         chk incompativel 'não tem pacote para esta arquitetura' file:///tmp/osmb-test/m.toml
chk http-recusado 'recusada' http://deb.debian.org/m.toml
chk dns-falha 'sem conexão' https://nao-existe.osmarcio.invalid/m.toml
chk tls-falha 'falha TLS' https://expired.badssl.com/
chk resposta-invalida 'manifesto inválido|erro HTTP' https://example.com/
# Sem internet: remove as rotas padrão IPv4 e IPv6 e restaura em seguida.
# (sem o campo "expires": rotas aprendidas por RA não podem ser recriadas com ele)
GW4=$(ip -4 route show default | head -1); GW6=$(ip -6 route show default | head -1 | sed -E 's/ (expires|nhid) [^ ]+//g')
ip -4 route del default 2>/dev/null; ip -6 route del default 2>/dev/null
chk sem-internet 'sem conexão' https://deb.debian.org/debian/m.toml
[[ -n $GW4 ]] && ip -4 route add $GW4; [[ -n $GW6 ]] && ip -6 route add $GW6
restore_conf
networkctl reconfigure $(ls /sys/class/net | grep -v '^lo$') >/dev/null 2>&1 || true
sleep 3
[[ -z $(dpkg --verify osmarcio-browser 2>&1) ]] && ok "limpeza conffile do pacote restaurado (dpkg --verify sem alterações)" || bad "limpeza $(dpkg --verify osmarcio-browser 2>&1 | head -2)"
ip -6 route show default | grep -q default && ok "limpeza rota IPv6 padrão restaurada" || echo "INFO sem rota IPv6 padrão (rede sem IPv6?)"
curl -fsS -o /dev/null https://deb.debian.org/debian/ && ok "limpeza rede restaurada (HTTPS ok)" || bad "limpeza rede não voltou"
;;
*) echo "fase desconhecida: $PHASE"; exit 2 ;;
esac
echo "RESULTADO($PHASE): $PASS passaram, $FAIL falharam"
[[ $FAIL -eq 0 ]]
