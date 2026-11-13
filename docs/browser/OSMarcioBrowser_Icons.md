# OSMarcio Browser — Ícone e integração com o desktop

## Design
Ícone **original** do projeto (`apps/osmarcio-browser/assets/icons/osmarcio-browser.svg`):
quadrado arredondado azul-marinho da identidade OSMarcio, **globo** verde-água
com meridianos (navegador) e uma **órbita laranja com seta** (navegação).
Não usa nem imita os logotipos do Chromium, do Google Chrome ou de terceiros.
Traços grossos para ficar legível a 16 px; fundo próprio para funcionar em
tema claro e escuro.

## Arquivos
| Arquivo | Uso |
|---|---|
| `assets/icons/osmarcio-browser.svg` | fonte vetorial |
| `assets/icons/hicolor/{16,24,32,48,64,128,256}x…/apps/osmarcio-browser.png` | PNG RGBA com transparência (`make browser-icons`, rsvg-convert no contêiner) |
| `assets/icons/hicolor/scalable/apps/osmarcio-browser.svg` | tema de ícones |

Identificador único: **`osmarcio-browser`** (pacote, executável, `.desktop`, ícone).

## Integração
| Local | Como | Verificado |
|---|---|---|
| Barra de título / barra de tarefas / Alt+Tab | `Window::set_window_icon` (16/32) e `set_window_app_icon` (128/256) → `_NET_WM_ICON` | `xprop` mostra 16x16 e 128x128 (antes: só o 128x128 do Chromium); capturas |
| Menu de aplicativos | `/usr/share/applications/osmarcio-browser.desktop` com `Icon=osmarcio-browser` | captura do menu Internet |
| Painel (lançador) | `/etc/xdg/xfce4/panel/launcher-3/osmarcio-browser.desktop` | captura |
| Tela Sobre | PNG 128 embutido (`include_bytes!`) | captura |
| Pacote Debian | `/usr/share/icons/hicolor/*/apps/` (o *trigger* do gtk-update-icon-cache atualiza o cache) | autoteste `app-browser-icon` |

## Limitação conhecida — WM_CLASS
O app pede `wm_class_name/class = "osmarcio-browser"` no callback
`linux_window_properties` do CEF; o callback **é chamado** (verificado em log),
mas a janela fica sem `WM_CLASS` no X11 (CEF 156 + cef-rs 156.4.0, estilo
Alloy). Causa exata não confirmada (suspeita: a cópia de volta das `CefString`
da struct no cef-rs). Efeito: o XFCE não associa a janela ao `.desktop` por
classe — o ícone aparece mesmo assim porque vem de `_NET_WM_ICON`. Por isso o
`.desktop` não declara `StartupWMClass`.

Subprocessos do Chromium não têm janela, logo não exibem ícone algum.
