# Arquitetura dos aplicativos do desktop (0.3.0)

Diagrama: `diagrams/drawio/OSMarcioLinux_Arquitetura.drawio`, páginas **08–12**
(PNG em `diagrams/png/OSMarcioLinux_Arquitetura-p8…p12.png`).

## Camadas

```
LightDM → Xorg (X11, driver modesetting, virtio-gpu) → sessão XFCE 4.20
  xfwm4 (gerenciador de janelas)  ·  xfce4-panel  ·  xfdesktop
    ├─ OSMarcio Terminal    Rust · GTK4 · VTE         org.osmarcio.Terminal
    ├─ OSMarcio Browser     Rust · CEF Views (Chromium 156)
    ├─ OSMarcio Files       Rust · GTK4 · GIO         org.osmarcio.Files
    └─ Campo Minado         Rust · GTK4 (regras em lib) org.osmarcio.Minesweeper
           └─ osmarcio-core (config XDG, web, updates, sysinfo — sem GTK, testável)
```

| App | Toolkit (verificado no código) | Pacote | Ícone | Menu |
|---|---|---|---|---|
| Terminal | GTK4 + VTE | `osmarcio-terminal` | `osmarcio-terminal` (próprio, novo) | Sistema |
| Browser | CEF Views (não usa GTK) | `osmarcio-browser` | `osmarcio-browser` | Internet |
| Files | GTK4 + GIO | `osmarcio-files` | `osmarcio-files` | Sistema |
| Campo Minado | GTK4 | `osmarcio-minesweeper` | `osmarcio-minesweeper` | Jogos |

Padrões aplicados aos quatro: ícone próprio no mesmo estilo (fundo arredondado
azul-marinho, cores OSMarcio, ver `docs/applications/img/icones-osmarcio.png`),
`.desktop` validado com `desktop-file-validate`, `Icon=` igual ao nome do
pacote, preferências em `~/.config/osmarcio/<app>.toml` (gravação atômica),
logs com prefixo próprio (`osmb:`, `osmf:`, `osmm:`), regras separadas da UI.

## Aplicativos padrão (`/etc/xdg/mimeapps.list`)
`inode/directory` → OSMarcio Files · `http/https/text/html` → OSMarcio Browser ·
`text/plain` → Mousepad. Helpers do XFCE (`/etc/xdg/xfce4/helpers.rc`):
TerminalEmulator, WebBrowser e FileManager apontam para os apps OSMarcio.

## Conceitos (educacional)

**GTK4.** Um `gtk::Application` registra um *application id*, abre janelas
(`ApplicationWindow`) e roda o laço de eventos. Widgets são organizados em
contêineres (Box, Grid, Paned, Stack); listas grandes usam o modelo
**lista + fábrica** (`gio::ListStore` → `MultiSelection` → `ColumnView`/`GridView`
com `SignalListItemFactory`), que só cria widgets para as linhas visíveis.
Ações (`gio::SimpleAction`) ligam menus, botões e atalhos à mesma função.

**Janelas no Linux (X11).** O app *pede*; o gerenciador de janelas (xfwm4)
*decide*. O app anuncia limites em `WM_NORMAL_HINTS` (tamanho mínimo/máximo) e
o WM publica `_NET_WM_ALLOWED_ACTIONS` (maximizar, redimensionar…). Se
mínimo = máximo, o WM entende "tamanho fixo" e não oferece maximizar — foi o
defeito do Browser ([detalhes](../browser/OSMarcioBrowser_Window_Management.md)).

**Maximizar ≠ tela cheia.** Maximizar ocupa a *área útil* (`_NET_WORKAREA`:
a tela menos o painel) e mantém a barra de título; tela cheia
(`_NET_WM_STATE_FULLSCREEN`, F11) cobre a tela inteira, inclusive o painel.
São estados independentes: sair da tela cheia volta ao estado anterior.

**Separar regras e interface.** As regras ficam em módulos sem GTK
(`game.rs`, `fsops.rs`, `listing.rs`, `trash.rs`, `osmarcio-core`), com
testes que rodam sem sessão gráfica; os callbacks GTK só traduzem eventos em
chamadas e redesenham a partir do estado.

**Empacotar e registrar no desktop.** Um `.deb` é um `tar` com
`DEBIAN/control` (+ `conffiles`) e a árvore a instalar: binário em
`/usr/bin`, `.desktop` em `/usr/share/applications`, ícones em
`/usr/share/icons/hicolor/<tamanho>/apps/`. Os *triggers* do Debian atualizam
o cache de ícones e o banco de `.desktop` sozinhos. Ver `scripts/build/deb.sh`
e [Makefile_Commands](../development/Makefile_Commands.md).

**Depurar.** `RUST_LOG=info <app>` mostra os eventos; `xprop`/`xwininfo`
mostram o que o WM vê; `journalctl -b` e `~/.xsession-errors` guardam erros da
sessão. Ver [Desktop_Troubleshooting](../troubleshooting/Desktop_Troubleshooting.md).
