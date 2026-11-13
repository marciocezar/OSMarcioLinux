# Campo Minado — guia do usuário

Abra pelo menu **OSMarcio → Jogos → Campo Minado** ou `osmarcio-minesweeper`.

**Objetivo:** abrir todas as casas sem mina. Cada número diz quantas minas há
nas 8 casas vizinhas. O primeiro clique nunca é uma mina (nem as vizinhas dele).

| Ação | Mouse | Teclado |
|---|---|---|
| Revelar casa | clique esquerdo | setas para mover, Enter ou Espaço |
| Marcar/desmarcar bandeira | clique direito | F (ou M) |
| Abrir vizinhas de um número | clique no número (se as bandeiras já somam o número) | Enter no número |
| Novo jogo | botão central ou menu | F2 ou Ctrl+N |
| Sobre | menu | F1 |

Menu ☰: Novo jogo · Fácil (9×9, 10) · Intermediário (16×16, 40) · Difícil
(30×16, 99) · Personalizado… · Recordes · Sobre.

O contador mostra minas − bandeiras (pode ficar negativo se houver bandeiras
demais). O tempo começa no primeiro clique. Os melhores tempos das três
dificuldades padrão ficam salvos em `~/.config/osmarcio/minesweeper.toml`
(partidas personalizadas não contam). A última dificuldade escolhida é lembrada.
