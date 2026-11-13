# Solução de problemas do desktop

| Sintoma | Verificar | Solução |
|---|---|---|
| Browser sem botão Maximizar (≤ 0.2.1) | `xprop WM_NORMAL_HINTS` mostra mínimo = máximo | atualize para 0.3.0 ([detalhes](../browser/OSMarcioBrowser_Window_Management.md)) |
| Browser maximizado com área vazia/preta | `RUST_LOG=info osmarcio-browser` e procure `janela WxH maximizada=1` | se o evento não aparece, o WM não aplicou; reporte com `xprop -id <janela> _NET_WM_STATE` |
| Abas do Terminal com "…" | normal com títulos longos; passe o mouse para ver o título completo | — |
| Files: "permissão negada" | `ls -ld <pasta>` | sua conta não tem acesso; o Files não contorna permissões |
| Files: "Nenhum aplicativo abre…" | `xdg-mime query default <tipo>` | instale um app para o tipo (ex.: um leitor de PDF) |
| Files não abre como root | intencional | use sua conta; para administração, terminal com sudo |
| Pastas Documentos/Downloads ausentes na barra lateral | `cat ~/.config/user-dirs.dirs` | `xdg-user-dirs-update` |
| Ícones genéricos nos apps | `ls /usr/share/icons/hicolor/48x48/apps/osmarcio-*` | `sudo gtk-update-icon-cache -f /usr/share/icons/hicolor` |
| Menu sem ícones | `xfconf-query -c xsettings -p /Gtk/MenuImages` | `true` (padrão da imagem) |
| Campo Minado não lembra recordes | `cat ~/.config/osmarcio/minesweeper.toml` | permissões da pasta `~/.config/osmarcio` |

## Logs úteis
```bash
RUST_LOG=info osmarcio-files ~          # eventos "osmf:"
RUST_LOG=info osmarcio-minesweeper      # eventos "osmm:"
RUST_LOG=info osmarcio-browser          # eventos "osmb:"
osmarcio-terminal                       # "abas=N botoes_fechar=…"
cat ~/.xsession-errors; journalctl -b -p warning
xprop | grep -E 'WM_CLASS|_NET_WM_STATE'    # clique na janela a inspecionar
```
