# OSMarcio Terminal (`osmarcio-terminal`)

Terminal gráfico em Rust. A emulação (sequências VT/ANSI, PTY, scrollback,
seleção) é da **VTE** — a mesma do GNOME Terminal; o app cuida de janela,
abas, atalhos e preferências.

## Estrutura do código

```
crates/osmarcio-core/src/
  tabs.rs      regras de abas (sem GTK) — política da última aba, ciclo Ctrl+Tab
  config.rs    leitura/gravação XDG atômica (tmp + rename)
apps/osmarcio-terminal/src/
  main.rs      gtk::Application, carrega a configuração
  config.rs    TerminalConfig (fonte, shell, scrollback, cursor, tema) e paletas
  window.rs    janela: Notebook de vte::Terminal, atalhos, menu da aba
```

**Modelo de estado.** `TermWindow` é um `Rc<Inner>`; todos os callbacks
GTK guardam só `Weak` (evita ciclos). Quem mantém o `Rc` vivo é o handler
`destroy` da janela — quando a janela morre, o handler é liberado e a
memória também. (Um bug real desse ponto foi pego pelo teste de GUI: sem
isso, o `Rc` morria logo após criar a janela e nenhum atalho funcionava.)

## Funcionalidades

| Recurso | Como |
|---|---|
| Shell | `$SHELL` do usuário, ou o configurado, ou `/bin/bash` |
| Nova aba | **Ctrl+T** ou botão ➕ |
| Fechar aba | **Ctrl+W**, botão ✕, ou menu da aba |
| Alternar abas | **Ctrl+Tab** / **Ctrl+Shift+Tab**, Ctrl+PgDn / Ctrl+PgUp |
| Renomear / Fechar outras | clique direito ou duplo clique na aba |
| Reordenar | arrastar a aba |
| Copiar / colar | **Ctrl+Shift+C** / **Ctrl+Shift+V** |
| Zoom | Ctrl+= / Ctrl+- / Ctrl+0 |
| Cores ANSI, scrollback, cursor, fonte | VTE + `terminal.toml` |

**Regra da última aba:** com uma aba, o ✕ fica oculto e Ctrl+W não fecha
nada — a tecla segue para o Bash, onde Ctrl+W é "apagar palavra". Com duas
ou mais abas os ✕ reaparecem. Digitar `exit` na última aba encerra a
janela (ação explícita do usuário).

Como Ctrl+T/Ctrl+W também são atalhos do readline, quem preferir o estilo
GNOME pode usar `plain_ctrl_tab_shortcuts = false` (passa a exigir Shift).

## Configuração — `~/.config/osmarcio/terminal.toml`

```toml
font = "Monospace 11"
shell = ""                      # vazio = $SHELL
scrollback_lines = 10000
cursor_shape = "block"          # block | ibeam | underline
cursor_blink = true
theme = "osmarcio-dark"         # osmarcio-dark | osmarcio-light
plain_ctrl_tab_shortcuts = true
```

Ainda não há janela de preferências: edite o arquivo (lido ao abrir).

## Build, testes e pacote

```bash
make terminal-build
make terminal-test       # unitários + teste de GUI real (Xvfb + xdotool)
make terminal-deb        # out/debs/osmarcio-terminal_0.1.0_arm64.deb
```

O teste de GUI abre o app num X virtual e verifica, por efeitos
observáveis: pipe + redirecionamento, histórico do Bash, Ctrl+C, Ctrl+W na
última aba, Ctrl+T, sessões independentes, Ctrl+Tab, fechamento com 2 abas
e `exit`. Resultado atual: 10/10.

## Limitações conhecidas

- Ainda não instalado em nenhuma imagem (o perfil Minimal não tem desktop).
- Testado só sob X11 (Xvfb); Wayland/labwc será testado na Fase 6.
- Sem janela de preferências, sem busca no scrollback, sem ícone próprio
  (usa `utilities-terminal` do tema).
- Nova aba abre no `$HOME`, não no diretório da aba atual.
