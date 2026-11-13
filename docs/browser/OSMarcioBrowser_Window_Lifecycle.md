# OSMarcio Browser — Ciclo de vida da janela

## Problema original
Clicar no **✕** da janela (e também Alt+F4 e Ctrl+Q) não fazia nada: a janela
continuava aberta, com 9 processos Chromium vivos. Reproduzido na VM com clique
real no botão da decoração do xfwm4 (XTest).

## Causa raiz
`WindowDelegate::can_close` fazia:

```rust
for h in hosts { h.close_browser(0); }   // pede o fechamento do browser…
0                                          // …e PROÍBE a janela de fechar
```

Com CEF Views, um browser hospedado numa `BrowserView` só termina de fechar
quando a janela que o contém é fechada: o CEF chama `DoClose`, que (no padrão)
pede de novo o fechamento da janela → `can_close` respondia **0** outra vez.
O browser esperava a janela e a janela esperava o browser: **impasse**.
O Ctrl+Q mostrava `atalho cmd=15` no log e nada mais.

Um segundo defeito apareceu ao corrigir o primeiro: **Ctrl+W numa janela com
várias abas fechava a janela inteira**, porque o `DoClose` padrão de uma
BrowserView fecha a janela-mãe.

## Correção (CEF 156, cef-rs 156.4.0)

```
X / Alt+F4 / Ctrl+Q / menu "Fechar janela"
  └─ CefWindowDelegate::can_close
       ├─ closing = true; fecha também o "Sobre"
       └─ para cada aba: host.try_close_browser()
            ├─ 0 → ainda precisa de um ciclo (beforeunload/DoClose); CEF chama can_close de novo
            └─ todas 1 → retorna 1: a janela fecha
  └─ LifeSpanHandler::do_close → 0 (janela fechando: deixa o CEF seguir)
  └─ LifeSpanHandler::on_before_close → remove a aba do estado
  └─ WindowDelegate::on_window_destroyed → solta referências (ui = None)
  └─ maybe_quit(): janela destruída + 0 abas + Sobre fechado → quit_message_loop()
main: run_message_loop() retorna → shutdown() → processo termina sem subprocessos
```

Fechar **uma aba** (Ctrl+W, ✕ da barra, menu "Fechar aba"):

```
close_browser(0) → do_close:
  ├─ várias abas → retira botão e BrowserView da interface, SOLTA as referências
  │                (só assim o CEF destrói o browser) e retorna 1 (não fecha a janela)
  └─ última aba  → closing = true e retorna 0 → mesmo caminho do X
```

O id do browser fica guardado na aba (`browser_id`, de `on_after_created`),
porque `BrowserView::browser()` deixa de responder durante a destruição.
Não há `kill`/encerramento forçado em nenhum caminho.

Arquivos: `apps/osmarcio-browser/src/ui.rs` (`can_close`, `on_window_destroyed`,
`on_do_close`, `on_browser_closed`, `maybe_quit`), `src/client.rs` (`do_close`,
`on_after_created`), `src/about.rs` (mesmo protocolo para a janela Sobre).

## Atalhos
| Ação | Efeito |
|---|---|
| ✕ da janela, Alt+F4, Ctrl+Q, ☰ → Fechar janela | fecha a janela (todas as abas) e encerra o app |
| Ctrl+W, ☰ → Fechar aba, ✕ da barra | fecha a aba; na última, fecha a janela |
| Ctrl+Shift+W | fecha as **outras** abas (comportamento já existente, mantido) |

**Múltiplas janelas:** o OSMarcio Browser tem uma janela principal por
processo (mais a janela Sobre). Uma nova execução de `osmarcio-browser` abre
outro processo independente; fechar um não afeta o outro.

## Testes (dentro do OSMarcio Linux, sessão XFCE real)
`make test-browser-hotfix` → `scripts/test/test-browser-hotfix.sh fechar`:
1 aba + X · 3 abas + Alt+F4 · página carregando + Ctrl+Q · download ativo + X ·
menu ☰ → Fechar janela · Ctrl+W com 2 abas (fecha só a aba) e na última aba ·
reabrir · X com o Sobre aberto. Em todos: `osmb: encerrado` no log e **0
processos** do navegador do usuário depois.
