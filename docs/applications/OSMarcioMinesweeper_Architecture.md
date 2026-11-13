# OSMarcio Minesweeper (Campo Minado) — arquitetura

Jogo em Rust + GTK4, executável `osmarcio-minesweeper`, pacote homônimo, app id
`org.osmarcio.Minesweeper`, menu **Jogos**. Diagrama: página 11.

## Separação regras × interface
| Arquivo | Conteúdo |
|---|---|
| `src/game.rs` (biblioteca) | `Config` (presets, validação), `Game`, `State`, `Cell`, `Rng` — **sem GTK** |
| `src/prefs.rs` (biblioteca) | última configuração e recordes (TOML em `~/.config/osmarcio/minesweeper.toml`) |
| `src/main.rs` | janela, botões do tabuleiro, temporizador, menus, diálogos |
`cargo test -p osmarcio-minesweeper --lib` roda os 18 testes sem sessão gráfica.

## Algoritmos
- **Primeiro clique seguro:** as minas só são sorteadas no 1º clique, fora da
  célula clicada **e das 8 vizinhas** (Fisher–Yates parcial sobre as casas
  livres). Por isso o máximo de minas é `linhas × colunas − 9`. Não há
  "redistribuição" depois: a regra é única e testada com 300 sementes no pior caso.
- **Vizinhança:** até 8 vizinhas com verificação de limites (canto 3, borda 5).
- **Abertura de áreas vazias:** *flood fill* **iterativo** com pilha
  (`VecDeque`) — sem recursão, testado no maior tabuleiro (24×40).
- **Chord:** clicar num número revelado abre as vizinhas se as bandeiras ao
  redor somam o número (bandeira errada = derrota, como no clássico).
- **Vitória:** reveladas = total − minas (as minas são marcadas). **Derrota:**
  todas as minas aparecem; a célula detonada fica vermelha; bandeiras erradas
  aparecem com ✗. Após o fim, o tabuleiro não aceita jogadas.
- **Aleatoriedade:** SplitMix64 semeado por `/dev/urandom`, sorteio sem viés
  (rejeição). Sem dependências externas.

## Estados
`NotStarted` → (1º clique) `Playing` → `Won` | `Lost` → (F2/Novo jogo) `NotStarted`.
O temporizador (GLib, 1 s) começa no 1º clique e para no fim (máx. 999 s).

## Dificuldades
Fácil 9×9/10 · Intermediário 16×16/40 · Difícil 30×16/99 · Personalizado
5–24 linhas, 5–40 colunas, 1…(linhas×colunas−9) minas (validado; erro explicado).

## Como implementar jogos em Rust (educacional)
Modele o jogo como uma `struct` com métodos que recebem ações (`reveal`,
`toggle_flag`, `chord`) e devolvem um resultado (`Outcome`); a interface chama
esses métodos e redesenha a partir do estado. Assim as regras são testáveis
como funções puras e a UI pode mudar sem tocar nelas.

## Teste automatizado da interface
`OSMARCIO_MINESWEEPER_SEED=<n>` fixa a semente (só para testes) e registra as
posições das minas no log após o 1º clique; `test-desktop-apps.sh` joga pelo
teclado (setas, Enter, F) e confere vitória, derrota, bandeiras, temporizador e
recorde.
