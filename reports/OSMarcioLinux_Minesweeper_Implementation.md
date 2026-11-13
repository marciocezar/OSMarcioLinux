# OSMarcio Minesweeper (Campo Minado) — implementação (0.3.0)

**Estado: Implementado e testado (ARM64, QEMU/HVF).**

## Entregue
`apps/osmarcio-minesweeper/` — Rust + GTK4; pacote `osmarcio-minesweeper_0.3.0_arm64.deb`;
ícone original; `.desktop` na categoria Jogos ("Campo Minado").
Arquitetura e algoritmos: [docs/applications/OSMarcioMinesweeper_Architecture.md](../docs/applications/OSMarcioMinesweeper_Architecture.md) ·
guia: [OSMarcioMinesweeper_User_Guide.md](../docs/applications/OSMarcioMinesweeper_User_Guide.md).

| Requisito | Implementação |
|---|---|
| Tabuleiro, minas aleatórias, revelar, contagem, flood fill | `game.rs`, flood fill iterativo (pilha) |
| Primeiro clique seguro (+ vizinhas) | minas sorteadas no 1º clique fora da área 3×3; máx. minas = L×C−9 |
| Bandeiras, chord, vitória, derrota, reinício | `toggle_flag`, `chord`, `State`, F2/Novo jogo |
| Fácil 9×9/10 · Intermediário 16×16/40 · Difícil 30×16/99 · Personalizado validado | `Config` |
| Contador, temporizador, indicador de resultado, Sobre, recordes XDG | `main.rs`, `prefs.rs` |
| Teclado | setas (navegação explícita linha/coluna), Enter/Espaço, F/M bandeira, F2, F1 |

## Testes
| Teste | Resultado |
|---|---|
| `make minesweeper-test` (sem sessão gráfica) | 18/18 — tamanhos, minas exatas, 1º clique seguro (300 sementes no pior caso), vizinhos e limites, revelar, flood fill (inclusive 24×40), bandeiras, vitória, derrota, chord, reinício, personalizados inválidos, RNG, recordes, TOML |
| `make test-desktop-apps` (VM, semente fixa, teclado real) | 10/10 em execuções consecutivas: 1º clique seguro com 16 minas em 5×5 e vitória; 10 minas no Fácil; bandeira marcar/desmarcar e contador; derrota numa mina conhecida com o temporizador parado; F2; vitória completa no Fácil; recorde gravado; fechar |
| Capturas | `reports/tests/screenshots/desktop-apps-arm64-*-{09,10,11}-minas-*.png` |

## Defeitos encontrados pelos testes e corrigidos
1. Navegação por setas do GTK (geométrica) parava após revelar casas → navegação explícita por linha/coluna.
2. Contraste invertido (casas fechadas brancas) → fechadas em relevo azul-acinzentado, abertas claras.
3. Cores dos números anuladas por especificidade CSS → seletores `.ms-cell.open.nN`.

## Limitações
Clique do mouse no tabuleiro não é automatizado (coberto pelas mesmas funções do teclado);
sem tema escuro próprio; recordes só para as 3 dificuldades padrão.
