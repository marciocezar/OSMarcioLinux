# OSMarcio Terminal — interface das abas (0.3.0)

## Problema
Os rótulos das abas só tinham largura **máxima** (24 caracteres) e nenhuma
mínima: com várias abas o GTK os encolhia até restar "…", e o padding padrão
do tema deixava a área clicável pequena.

## Implementação (`apps/osmarcio-terminal/src/window.rs`)
- Rótulo com `width_chars(18)` (≈160 px na fonte padrão) e `max_width_chars(28)`,
  reticências no fim e **tooltip com o título completo** (atualizado junto com
  o título que o shell envia por escape OSC 0).
- CSS próprio (`notebook.osm-tabs`), aplicado só às abas: padding 5 px × 14 px,
  altura mínima 30 px, aba ativa em negrito, botão ✕ com área de 22 px.
  A fonte do conteúdo do shell **não muda**.
- Medidas em caracteres, não pixels: acompanham fonte e escala. Não há largura
  fixa de janela; com muitas abas o `GtkNotebook` (scrollable) mostra setas de
  rolagem e mantém a aba ativa visível; o botão "nova aba" fica no cabeçalho,
  sempre acessível.

## Regras preservadas (`osmarcio_core::tabs::TabPolicy`)
2+ abas → cada uma tem ✕ e fecha em qualquer ordem; 1 aba → ✕ oculto e
Ctrl+Shift+W não fecha; o botão de fechar a janela continua independente.

## Como funciona (educacional)
Cada aba é uma página do `GtkNotebook` cujo conteúdo é um `vte::Terminal`. O
VTE cria um **pseudoterminal (PTY)** e inicia o shell nele (`spawn_async`):
cada aba tem seu próprio processo bash, independente das demais. O título vem
do shell (`window-title-changed`). Fechar uma aba encerra só aquele processo.

## Testes
- Contêiner (Xvfb): `make apps-test` → `test-terminal-gui.sh` 10/10 (última aba,
  Ctrl+W/Ctrl+T, sessões independentes, exit).
- VM (`make test-desktop-apps`): 1, 5 e 10 abas com capturas; fechamento 10→1;
  última aba protegida; maximizar (Alt+F10); redimensionar; fechar.
- **Não testado:** tema claro do GTK4 (a imagem usa o padrão Adwaita claro para
  GTK4 e Greybird-dark para GTK3; o CSS não depende de cores), telas de alta
  densidade (escala 2×).
