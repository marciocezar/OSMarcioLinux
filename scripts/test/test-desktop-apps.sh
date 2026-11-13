#!/bin/bash
# Testes gráficos dos apps do desktop — roda DENTRO do OSMarcio Linux (root),
# com a sessão XFCE do usuário osmarcio aberta em :0. Enviado e executado pelo
# host via qemu-guest-agent (scripts/test/test-desktop-apps-qemu.sh).
#
# uso: test-desktop-apps.sh <fase>
#   terminal-1 | terminal-5 | terminal-10 | terminal-fim
#   browser-max | browser-fim
#   files-a | files-cancelar | files-copia-inicio | files-copia-fim | files-b | files-icones | files-fim
#   minas-vitoria | minas-derrota | minas-recorde
#
# Tudo é feito com cliques/teclas reais (XTest) e verificado no sistema de
# arquivos ou nos logs dos apps. Arquivos de teste ficam só em ~/osmf-teste
# (criado e removido pelo próprio teste) — nenhum arquivo do usuário é tocado.
set -u
PHASE=${1:?fase}
U=osmarcio
H=/home/$U
T=$H/osmf-teste
X=(env DISPLAY=:0 XAUTHORITY=$H/.Xauthority)
PASS=0; FAIL=0
ok()  { echo "PASS $*"; PASS=$((PASS+1)); }
bad() { echo "FAIL $*"; FAIL=$((FAIL+1)); }
xdo() { "${X[@]}" xdotool "$@"; }
as_user() { runuser -u "$U" -- env DISPLAY=:0 XAUTHORITY=$H/.Xauthority HOME=$H XDG_RUNTIME_DIR=/run/user/1000 \
  DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus LANG=pt_BR.UTF-8 RUST_LOG=info "$@"; }
waitlog() { for _ in $(seq $(( $3 * 4 ))); do grep -Eq "$2" "$1" 2>/dev/null && return 0; sleep 0.25; done; return 1; }
mark() { wc -l < "$1" 2>/dev/null || echo 0; }
waitnew() { # waitnew <log> <linha inicial> <regex> <segundos> — só linhas novas
  for _ in $(seq $(( $4 * 4 ))); do tail -n +"$(( $2 + 1 ))" "$1" | grep -Eq "$3" && return 0; sleep 0.25; done; return 1; }
waitfs() { for _ in $(seq $(( $2 * 4 ))); do eval "$1" && return 0; sleep 0.25; done; return 1; }
win() { xdo search --name "$1" 2>/dev/null | tail -1; }
active_is() { # active_is <título> <segundos> — espera um diálogo ficar ativo
  for _ in $(seq $(( $2 * 4 ))); do [[ $(xdo getactivewindow getwindowname 2>/dev/null) == "$1"* ]] && return 0; sleep 0.25; done; return 1; }
geo() { "${X[@]}" xwininfo -id "$1" | awk '/Absolute upper-left X/{x=$4}/Absolute upper-left Y/{y=$4}/Width:/{w=$2}/Height:/{h=$2}END{print w"x"h"+"x"+"y}'; }
wstate() { "${X[@]}" xprop -id "$1" _NET_WM_STATE | grep -o 'MAXIMIZED_[A-Z]*\|FULLSCREEN' | tr '\n' ' '; }
row() { echo $(( 129 + 32 * $1 )); }            # linha i da lista do Files (janela em 0,0)
click_row() { xdo mousemove 400 "$(row "$1")" click 1; sleep 0.4; }
dbl_row() { xdo mousemove 400 "$(row "$1")" click --repeat 2 --delay 90 1; sleep 1; }
ntrash() { ls "$H/.local/share/Trash/files" 2>/dev/null | wc -l; }
L=/tmp/osm-apps; mkdir -p $L

case $PHASE in
# ------------------------------------------------------------------ Terminal
terminal-1)
  pkill -u $U osmarcio-termin; sleep 1
  as_user osmarcio-terminal > $L/term.log 2>&1 &
  waitlog $L/term.log 'abas=1' 15 && ok "terminal-1-aba abriu" || bad "terminal não abriu"
  W=$(xdo search --classname osmarcio-terminal | tail -1); echo "$W" > $L/term.w
  xdo windowactivate --sync "$W" windowmove "$W" 0 0 windowsize "$W" 900 560; sleep 1
  grep -q 'abas=1 botoes_fechar=false' $L/term.log && ok "terminal-ultima-aba sem botão X" || bad "terminal-ultima-aba"
  xdo key ctrl+shift+w; sleep 1
  [[ $(grep -c 'shell encerrado' $L/term.log) == 0 ]] && pgrep -u $U osmarcio-termin >/dev/null && ok "terminal-ultima-aba Ctrl+Shift+W não fecha a última aba" || bad "terminal fechou a última aba"
  ;;
terminal-5)
  W=$(cat $L/term.w); xdo windowactivate --sync "$W"
  for _ in 1 2 3 4; do xdo key ctrl+shift+t; sleep 0.6; done
  waitlog $L/term.log 'abas=5 botoes_fechar=true' 10 && ok "terminal-5-abas com botões X" || bad "terminal-5-abas"
  xdo type --delay 20 'printf "\033]0;Projeto OSMarcio\007"; PROMPT_COMMAND=; sleep 0'; xdo key Return; sleep 1
  ;;
terminal-10)
  W=$(cat $L/term.w); xdo windowactivate --sync "$W"
  for _ in 1 2 3 4 5; do xdo key ctrl+shift+t; sleep 0.6; done
  waitlog $L/term.log 'abas=10 ' 10 && ok "terminal-10-abas (faixa rola, aba ativa visível)" || bad "terminal-10-abas"
  ;;
terminal-fim)
  W=$(cat $L/term.w); xdo windowactivate --sync "$W"
  xdo key ctrl+Page_Up; sleep 0.3; xdo key ctrl+Page_Up; sleep 0.3
  for _ in $(seq 9); do xdo key ctrl+shift+w; sleep 0.5; done
  waitlog $L/term.log 'abas=1 botoes_fechar=false' 10 && ok "terminal-fechamento individual 10→1 (em ordem qualquer), X some na última" || bad "terminal-fechamento"
  xdo key ctrl+shift+w; sleep 1
  pgrep -u $U osmarcio-termin >/dev/null && ok "terminal-protecao última aba preservada" || bad "terminal-protecao"
  xdo key alt+F10; sleep 1.5
  [[ $(wstate "$W") == *MAXIMIZED_VERT* ]] && ok "terminal-maximizar $(geo "$W")" || bad "terminal-maximizar"
  xdo key alt+F10; sleep 1
  xdo windowsize "$W" 520 300; sleep 1
  ok "terminal-redimensionar $(geo "$W")"
  xdo key alt+F4; sleep 1.5
  pgrep -u $U osmarcio-termin >/dev/null && bad "terminal não fechou pela janela" || ok "terminal-fechar janela fechada"
  ;;
# ------------------------------------------------------------------ Browser
browser-max)
  pkill -u $U -f osmarcio-browser; sleep 1
  as_user osmarcio-browser https://example.com > $L/br.log 2>&1 &
  waitlog $L/br.log 'load_end status=200' 40 && ok "browser-https example.com" || bad "browser-https"
  sleep 1; W=$(win 'OSMarcio Browser'); echo "$W" > $L/br.w
  xdo windowactivate --sync "$W"
  n=$(geo "$W"); echo "$n" > $L/br.normal
  "${X[@]}" xprop -id "$W" _NET_WM_ALLOWED_ACTIONS | grep -q MAXIMIZE_VERT && ok "browser-acao-maximizar anunciada ao xfwm4" || bad "browser sem ação maximizar"
  wa=$("${X[@]}" xprop -root _NET_WORKAREA | awk -F'[=,]' '{gsub(/ /,""); print $4"x"$5}')
  xdo key alt+F10; sleep 2.5
  g=$(geo "$W"); st=$(wstate "$W")
  ww=${g%%x*}; rest=${g#*x}; hh=${rest%%+*}; y=${g##*+}
  [[ $st == *MAXIMIZED_VERT*MAXIMIZED_HORZ* || $st == *MAXIMIZED_HORZ*MAXIMIZED_VERT* ]] && [[ $ww == "${wa%x*}" && $((hh + y)) == "${wa#*x}" ]] \
    && ok "browser-maximizar $g = área útil ${wa} (painel preservado)" || bad "browser-maximizar $g [$st] área útil $wa"
  # (Window::is_maximized() do CEF devolve 0 sob o xfwm4; o tamanho é o que importa.)
  waitlog $L/br.log "janela ${ww}x${hh}" 5 && ok "browser-cef recebeu o novo tamanho: $(grep -o "janela ${ww}x${hh}[^ ]*" $L/br.log | tail -1)" || bad "browser-cef sem evento de tamanho"
  ;;
browser-fim)
  W=$(cat $L/br.w); xdo windowactivate --sync "$W"
  xdo key alt+F10; sleep 2
  [[ $(geo "$W") == "$(cat $L/br.normal)" && -z $(wstate "$W" | tr -d ' ') ]] && ok "browser-restaurar $(geo "$W")" || bad "browser-restaurar $(geo "$W") esperado $(cat $L/br.normal)"
  xdo key alt+F10; sleep 2
  xdo key ctrl+t; sleep 1; xdo key ctrl+l; xdo type --delay 15 'https://www.debian.org/'; xdo key Return
  waitlog $L/br.log 'load_end status=200 url=https://www.debian.org/' 40 && ok "browser-abas nova aba carregou maximizada" || bad "browser-abas"
  xdo key ctrl+Tab; sleep 0.5; xdo key Next Next; sleep 0.5
  xdo key F11; sleep 2
  [[ $(wstate "$W") == *FULLSCREEN* ]] && ok "browser-tela-cheia F11 $(geo "$W")" || bad "browser-tela-cheia"
  xdo key F11; sleep 2
  [[ $(wstate "$W") == *MAXIMIZED_VERT* ]] && ok "browser-tela-cheia sair volta a maximizada (independentes)" || bad "browser sair da tela cheia [$(wstate "$W")]"
  xdo windowsize "$W" 700 500 2>/dev/null; sleep 1.5
  xdo key ctrl+q
  for _ in $(seq 40); do pgrep -u $U -f /usr/lib/osmarcio-browser/osmarcio-browser >/dev/null || break; sleep 0.25; done
  pgrep -u $U -f /usr/lib/osmarcio-browser/osmarcio-browser >/dev/null && bad "browser-fechar processos restantes" || ok "browser-fechar encerrado sem órfãos"
  as_user osmarcio-browser https://example.com > $L/br2.log 2>&1 &
  waitlog $L/br2.log 'load_end status=200' 40 && ok "browser-reabrir" || bad "browser-reabrir"
  xdo key --window "$(win 'OSMarcio Browser')" ctrl+q 2>/dev/null; sleep 0.5; xdo windowactivate --sync "$(win 'OSMarcio Browser')" key ctrl+q; sleep 3
  ;;
# ------------------------------------------------------------------ Files
files-a)
  pkill -u $U osmarcio-files; sleep 0.5
  rm -rf "$T"
  as_user sh -c "mkdir -p '$T/Destino' && echo 'conteúdo A' > '$T/a.txt' && dd if=/dev/zero of='$T/grande.bin' bs=1M count=2048 status=none && printf '#!/bin/sh\ntouch /tmp/NAO-DEVIA-EXECUTAR\n' > '$T/script.sh' && chmod +x '$T/script.sh' && echo nota > '$T/nota.txt' && touch '$T/.oculto'"
  rm -f /tmp/NAO-DEVIA-EXECUTAR
  as_user osmarcio-files "$T" > $L/files.log 2>&1 &
  waitlog $L/files.log 'osmf: janela criada' 15 && ok "files-abrir $T" || bad "files não abriu"
  W=$(win 'OSMarcio Files'); echo "$W" > $L/files.w
  xdo windowactivate --sync "$W" windowmove "$W" 0 0 windowsize "$W" 1000 640; sleep 1
  grep -q "pasta $T itens=5" $L/files.log && ok "files-listar 5 itens (oculto escondido)" || bad "files-listar $(grep -o 'pasta .*' $L/files.log | tail -1)"
  # linhas: 0 Destino · 1 a.txt · 2 grande.bin · 3 nota.txt · 4 script.sh
  click_row 1; xdo key ctrl+shift+n; active_is "Nova pasta" 5; xdo key ctrl+a; xdo type --delay 20 'Pasta Nova'; xdo key Return
  waitfs "[[ -d '$T/Pasta Nova' ]]" 5 && ok "files-nova-pasta criada" || bad "files-nova-pasta"
  # linhas: 0 Destino · 1 Pasta Nova · 2 a.txt · 3 grande.bin · 4 nota.txt · 5 script.sh
  click_row 2; xdo key ctrl+c; dbl_row 0; m=$(mark $L/files.log); xdo key ctrl+v
  waitnew $L/files.log "$m" 'tarefa ok' 15   # espera a UI terminar e recarregar
  waitfs "[[ -f '$T/Destino/a.txt' && -f '$T/a.txt' ]]" 10 && ok "files-copiar a.txt → Destino (origem preservada)" || bad "files-copiar"
  xdo key alt+Left; sleep 1
  click_row 2; xdo key ctrl+x; dbl_row 1; m=$(mark $L/files.log); xdo key ctrl+v
  waitnew $L/files.log "$m" 'tarefa ok' 15
  waitfs "[[ -f '$T/Pasta Nova/a.txt' && ! -e '$T/a.txt' ]]" 10 && ok "files-mover a.txt → Pasta Nova" || bad "files-mover"
  # (nomes Unicode são cobertos pelos testes unitários; o xdotool digita ASCII com segurança)
  click_row 0; xdo key F2; active_is "Renomear" 5; sleep 0.3; xdo type --delay 30 'relatorio-final'; xdo key Return
  waitfs "[[ -f '$T/Pasta Nova/relatorio-final.txt' && ! -e '$T/Pasta Nova/a.txt' ]]" 5 && ok "files-renomear a.txt → relatorio-final.txt (extensão preservada)" || bad "files-renomear $(ls "$T/Pasta Nova")"
  t0=$(ntrash); click_row 0; xdo key Delete
  waitfs "[[ ! -e '$T/Pasta Nova/relatorio-final.txt' && \$(ntrash) -gt $t0 ]]" 5 && ok "files-lixeira relatorio-final.txt enviado à lixeira" || bad "files-lixeira"
  xdo key ctrl+l; xdo type --delay 20 'lixeira:'; xdo key Return; sleep 1.5
  click_row 0; xdo key ctrl+shift+r
  waitfs "[[ -f '$T/Pasta Nova/relatorio-final.txt' ]]" 5 && ok "files-restaurar da lixeira ao local original" || bad "files-restaurar"
  xdo key ctrl+l; xdo type --delay 10 "$T"; xdo key Return; sleep 1
  ;;
files-cancelar)
  W=$(cat $L/files.w); xdo windowactivate --sync "$W"
  # linhas: 0 Destino · 1 Pasta Nova · 2 grande.bin · 3 nota.txt · 4 script.sh
  m=$(mark $L/files.log)
  click_row 2; xdo key ctrl+c; dbl_row 0; xdo key ctrl+v; sleep 0.3; xdo key Escape
  waitnew $L/files.log "$m" 'tarefa falhou: .*cancelada' 20 && ok "files-cancelar cópia de 2 GiB cancelada" || bad "files-cancelar $(tail -n +$((m+1)) $L/files.log | grep -o 'tarefa .*' | tail -1)"
  [[ -z $(ls "$T/Destino" | grep grande) ]] && ok "files-cancelar sem arquivo parcial no destino" || bad "files-cancelar deixou $(ls "$T/Destino")"
  ;;
files-copia-inicio)
  W=$(cat $L/files.w); xdo windowactivate --sync "$W"
  mark $L/files.log > $L/files.mark
  xdo key ctrl+v; sleep 0.5
  ;;
files-copia-fim)
  waitnew $L/files.log "$(cat $L/files.mark)" 'tarefa ok: 1 item' 120 && ok "files-copia-grande concluída" || bad "files-copia-grande"
  [[ $(stat -c %s "$T/Destino/grande.bin" 2>/dev/null) == $(stat -c %s "$T/grande.bin") ]] && ok "files-copia-grande tamanho idêntico (2 GiB)" || bad "files-copia-grande tamanho"
  W=$(cat $L/files.w); xdo windowactivate --sync "$W"; xdo key alt+Left; sleep 1
  ;;
files-b)
  W=$(cat $L/files.w); xdo windowactivate --sync "$W"
  xdo key ctrl+l; xdo type --delay 20 '/root'; xdo key Return
  waitlog $L/files.log 'abrir /root: permissão negada' 5 && ok "files-permissao /root negada (usuário comum)" || bad "files-permissao"
  sleep 1; xdo key Return; sleep 0.5
  xdo windowactivate --sync "$W"; click_row 2; xdo key alt+Return; sleep 1.5
  P=$(win 'Propriedades — grande.bin'); [[ -n $P ]] && ok "files-propriedades janela aberta" || bad "files-propriedades"
  echo "$P" > $L/props.w
  ;;
files-icones)
  [[ -n $(cat $L/props.w) ]] && xdo windowactivate --sync "$(cat $L/props.w)" key alt+F4; sleep 0.5
  W=$(cat $L/files.w); xdo windowactivate --sync "$W"
  xdo key ctrl+f; sleep 0.5; xdo type --delay 30 'grande'; sleep 1.5
  waitlog $L/files.log 'busca resultados=1' 5 && ok "files-busca 'grande' → 1 resultado" || bad "files-busca"
  xdo key Escape; sleep 1
  n0=$(grep -o "pasta $T itens=[0-9]*" $L/files.log | tail -1 | cut -d= -f2)
  xdo key ctrl+h; sleep 1
  n1=$(grep -o "pasta $T itens=[0-9]*" $L/files.log | tail -1 | cut -d= -f2)
  [[ $n1 == $((n0 + 1)) ]] && ok "files-ocultos Ctrl+H mostra .oculto ($n0→$n1)" || bad "files-ocultos $n0→$n1"
  xdo key ctrl+h; sleep 1
  # linhas: 0 Destino · 1 Pasta Nova · 2 grande.bin · 3 nota.txt · 4 script.sh
  dbl_row 3; waitfs "pgrep -u $U mousepad >/dev/null" 15 && ok "files-abrir nota.txt abriu no editor padrão (mousepad, via MIME)" || bad "files-abrir texto"
  pkill -u $U mousepad; sleep 0.5; xdo windowactivate --sync "$W"
  dbl_row 4; sleep 1.5
  grep -q 'execução recusada' $L/files.log && [[ ! -e /tmp/NAO-DEVIA-EXECUTAR ]] && ok "files-seguranca script.sh não executado" || bad "files-seguranca"
  xdo key Return; sleep 0.5; xdo windowactivate --sync "$W"
  xdo key ctrl+1; sleep 1
  ;;
files-fim)
  W=$(cat $L/files.w); xdo windowactivate --sync "$W"; xdo key ctrl+2; sleep 1
  t0=$(ntrash); click_row 3; xdo key shift+Delete; sleep 1.2; xdo key Tab; sleep 0.2; xdo key Return
  waitfs "[[ ! -e '$T/nota.txt' ]]" 5 && [[ $(ntrash) == "$t0" ]] && ok "files-excluir-permanente com confirmação (não foi à lixeira)" || bad "files-excluir-permanente"
  xdo key alt+F4; sleep 1
  pgrep -u $U osmarcio-files >/dev/null && bad "files não fechou" || ok "files-fechar janela"
  rm -rf "$T"
  ;;
# ------------------------------------------------------------------ Campo Minado
minas-vitoria)
  pkill -u $U -f osmarcio-minesweeper; sleep 0.5
  as_user sh -c 'mkdir -p ~/.config/osmarcio && printf "rows = 5\ncols = 5\nmines = 16\n" > ~/.config/osmarcio/minesweeper.toml'
  as_user env OSMARCIO_MINESWEEPER_SEED=11 osmarcio-minesweeper > $L/ms.log 2>&1 &
  waitlog $L/ms.log 'janela criada' 15 || bad "campo minado não abriu"
  W=$(win 'Campo Minado'); echo "$W" > $L/ms.w; xdo windowactivate --sync "$W" windowmove "$W" 0 0; sleep 1
  xdo key Right Right Down Down Return; sleep 1
  grep -q 'revelar 2,2' $L/ms.log && ok "minas-primeiro-clique (2,2) seguro com 16 minas em 25 casas" || bad "minas-primeiro-clique"
  grep -q 'fim vitoria=true' $L/ms.log && ok "minas-vitoria área 3×3 livre = vitória (personalizado 5×5/16)" || bad "minas-vitoria"
  ;;
minas-derrota)
  pkill -u $U -f osmarcio-minesweeper; sleep 0.5
  as_user sh -c 'printf "rows = 9\ncols = 9\nmines = 10\n" > ~/.config/osmarcio/minesweeper.toml'
  as_user env OSMARCIO_MINESWEEPER_SEED=7 osmarcio-minesweeper > $L/ms.log 2>&1 &
  waitlog $L/ms.log 'janela criada' 15
  W=$(win 'Campo Minado'); echo "$W" > $L/ms.w; xdo windowactivate --sync "$W" windowmove "$W" 0 0; sleep 1
  for _ in 1 2 3 4; do xdo key Right Down; done; xdo key Return; sleep 2.5
  mines=$(grep -o 'minas(teste)=[0-9,;]*' $L/ms.log | cut -d= -f2); echo "$mines" > $L/ms.mines
  [[ $(echo "$mines" | tr ';' '\n' | grep -c ,) == 10 ]] && ok "minas-quantidade exatamente 10 minas (Fácil 9×9)" || bad "minas-quantidade $mines"
  m=$(echo "$mines" | cut -d';' -f1); mr=${m%,*}; mc=${m#*,}
  # foco em (4,4) → mina (mr,mc)
  for _ in $(seq $((4 - mr))); do xdo key Up; done; for _ in $(seq $((mc - 4))); do xdo key Right; done
  for _ in $(seq $((4 - mc))); do xdo key Left; done
  xdo key f; sleep 0.5
  grep -q "bandeira $mr,$mc minas_restantes=9" $L/ms.log && ok "minas-bandeira marcar (contador 9)" || bad "minas-bandeira"
  xdo key f; sleep 0.5
  grep -q "bandeira $mr,$mc minas_restantes=10" $L/ms.log && ok "minas-bandeira desmarcar (contador 10)" || bad "minas-desmarcar"
  xdo key Return; sleep 1
  grep -q 'fim vitoria=false tempo=[1-9]' $L/ms.log && ok "minas-derrota mina em ($mr,$mc); temporizador parou em $(grep -o 'fim vitoria=false tempo=[0-9]*' $L/ms.log | cut -d= -f3) s" || bad "minas-derrota $(grep 'fim' $L/ms.log)"
  ;;
minas-recorde)
  W=$(cat $L/ms.w); xdo windowactivate --sync "$W"
  xdo key F2; sleep 1
  grep -q 'novo jogo' $L/ms.log && ok "minas-reiniciar F2 novo jogo" || bad "minas-reiniciar"
  # Mesma semente + mesmo primeiro clique (4,4) = mesmo tabuleiro da fase anterior.
  for _ in $(seq 9); do xdo key Up Left; done
  for _ in 1 2 3 4; do xdo key Right Down; done; xdo key Return; sleep 0.5
  for _ in $(seq 9); do xdo key Up Left; done
  mines=";$(cat $L/ms.mines);"
  for r in $(seq 0 8); do
    for c in $(seq 0 8); do
      [[ $mines == *";$r,$c;"* ]] || xdo key Return
      [[ $c -lt 8 ]] && xdo key Right
    done
    xdo key Down; for _ in $(seq 8); do xdo key Left; done
  done
  sleep 1
  grep -q 'fim vitoria=true' $L/ms.log && ok "minas-vitoria Fácil completo pelo teclado" || bad "minas-vitoria Fácil"
  grep -q '^best_easy = [0-9]' $H/.config/osmarcio/minesweeper.toml && ok "minas-recorde salvo em ~/.config/osmarcio/minesweeper.toml ($(grep best_easy $H/.config/osmarcio/minesweeper.toml))" || bad "minas-recorde"
  ;;
minas-fim)
  W=$(cat $L/ms.w); xdo windowactivate --sync "$W"; xdo key alt+F4; sleep 1
  pgrep -u $U -f osmarcio-minesweeper >/dev/null && bad "campo minado não fechou" || ok "minas-fechar janela"
  rm -f $H/.config/osmarcio/minesweeper.toml
  ;;
*) echo "fase desconhecida: $PHASE"; exit 2 ;;
esac
echo "RESULTADO($PHASE): $PASS passaram, $FAIL falharam"
[[ $FAIL -eq 0 ]]
