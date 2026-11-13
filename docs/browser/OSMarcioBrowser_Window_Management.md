# OSMarcio Browser — maximizar, restaurar e tela cheia (0.3.0)

## Problema
A janela não maximizava: o xfwm4 não mostrava o botão Maximizar, Alt+F10 não
fazia nada e a borda não redimensionava.

## Causa raiz (medida na VM com `xprop`)
```
WM_NORMAL_HINTS: minimum size 1100 by 760 · maximum size 1100 by 760
_NET_WM_ALLOWED_ACTIONS: … sem MAXIMIZE_HORZ/VERT e sem RESIZE
```
Duas divergências entre o **cef-rs 156.4.0** e o CEF em C++:
1. `WindowDelegate::can_resize/can_maximize/can_minimize` têm implementação
   padrão `Default::default()` = **0** no cef-rs (no C++ o padrão é `true`).
2. Sem `minimum_size`/`maximum_size` no delegate, o CEF Views usa o tamanho
   preferido (1100×760) como mínimo **e** máximo.
Resultado: o WM recebia "janela de tamanho fixo". É a mesma família de defeito
do `BoxLayoutSettings::default()` (página em branco, corrigido na 0.2.0).

## Correção (`apps/osmarcio-browser/src/ui.rs`)
```rust
fn minimum_size(..) -> Size { Size { width: 520, height: 360 } }  // usabilidade
fn maximum_size(..) -> Size { Size { width: 0, height: 0 } }      // sem limite
fn can_resize(..) -> i32 { 1 }  fn can_maximize(..) -> i32 { 1 }  fn can_minimize(..) -> i32 { 1 }
fn on_window_bounds_changed(..)  // registra tamanho, maximizada e tela cheia
```
Nada de tamanho fixo da tela do desenvolvedor: quem decide a área é o WM
(`_NET_WORKAREA`). 1100×760 continua só como tamanho inicial "restaurado".

## Como o CEF acompanha o redimensionamento (educacional)
O WM muda a janela X11 → o Chromium (Aura) recebe o novo tamanho → o CEF Views
refaz o layout da `Window` → o `BoxLayout` vertical (eixo cruzado STRETCH,
`BrowserView` com flex 1) entrega a área restante à página → o renderer recebe
a nova viewport e redesenha. Não há código do app para "repassar" o tamanho.

## Tela cheia
F11 chama `Window::set_fullscreen` (estado `_NET_WM_STATE_FULLSCREEN`), separado
da maximização: maximizada → F11 → F11 volta a maximizada.

## Testes (`make test-desktop-apps`, sessão XFCE real, 1280×800, painel visível)
Ação Maximizar anunciada · Alt+F10 → janela = área útil (1280×763 contando a
barra de título) · evento `maximizada=1` do CEF · captura com conteúdo desenhado
além dos 1100 px antigos · restaurar volta exatamente ao tamanho anterior ·
nova aba carrega HTTPS maximizada · troca de abas e rolagem · F11 entra/sai ·
redimensionar à mão · Ctrl+Q sem órfãos · reabrir. Clique real no botão "+" da
decoração verificado durante o diagnóstico.
**Não testado:** escala de exibição 2× e outras resoluções além de 1280×800.
