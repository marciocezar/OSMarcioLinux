# OSMarcio Browser — correção da maximização (0.3.0)

**Estado: Implementado e testado (ARM64, QEMU/HVF, XFCE/xfwm4, 1280×800).**

## Diagnóstico (VM, `xprop`/`xwininfo`)
```
WM_NORMAL_HINTS: program specified minimum size: 1100 by 760
                 program specified maximum size: 1100 by 760
_NET_WM_ALLOWED_ACTIONS: CLOSE ABOVE BELOW FULLSCREEN MOVE SHADE MINIMIZE … (sem MAXIMIZE, sem RESIZE)
```
A decoração do xfwm4 não tinha botão Maximizar; Alt+F10 e as bordas não faziam nada.

## Causa raiz
No **cef-rs 156.4.0**, `WindowDelegate::can_resize/can_maximize/can_minimize`
retornam `Default::default()` = **0** (no CEF C++ o padrão é `true`), e sem
`minimum_size`/`maximum_size` o CEF Views usa o tamanho preferido como mínimo e
máximo. Mesma classe de divergência do `BoxLayoutSettings` corrigido na 0.2.0.

## Correção (`apps/osmarcio-browser/src/ui.rs`, `about.rs`)
`can_resize/can_maximize/can_minimize = 1`; `minimum_size` 520×360;
`maximum_size` 0×0 (sem limite); `on_window_bounds_changed` registra o tamanho.
Nenhuma dimensão fixa de tela; nada de forçar geometria — o WM decide a área
(`_NET_WORKAREA`) e o layout do CEF (BrowserView flex, STRETCH) acompanha.
Detalhes: [docs/browser/OSMarcioBrowser_Window_Management.md](../docs/browser/OSMarcioBrowser_Window_Management.md).

## Testes
| Cenário | Resultado |
|---|---|
| Ação Maximizar anunciada (`_NET_WM_ALLOWED_ACTIONS`) | PASS |
| Clique real no botão "+" da decoração (diagnóstico) | 1280×740+0+23, `MAXIMIZED_HORZ/VERT` |
| Alt+F10 = área útil (1280×763 = tela − painel, contando a barra de título) | PASS |
| CEF recebeu o novo tamanho (`janela 1280x740…`) | PASS |
| Renderização até a borda (barra de ferramentas e rolagem da página; 18 cores vs. 1 da janela antiga) | PASS |
| Restaurar → 1100×760 exatos | PASS |
| Nova aba carrega HTTPS maximizada; troca de abas; rolagem | PASS |
| F11 → tela cheia 1280×800; F11 → volta maximizada | PASS |
| Redimensionar à mão (700×500, 800×500) | PASS |
| Ctrl+Q sem processos órfãos; reabrir | PASS |
| Regressão do hotfix 0.2.1 (`make test-browser-hotfix`) | ver relatório de regressão |

## Observação
`Window::is_maximized()` do CEF retorna 0 sob o xfwm4 mesmo maximizada (o
estado X11 não é refletido); usado só no log, não afeta o comportamento.

## Não testado
Escala de exibição 2×, outras resoluções, Wayland, x86_64.
