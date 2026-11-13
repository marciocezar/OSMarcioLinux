# OSMarcio Browser — Arquitetura

Navegador do OSMarcio Linux escrito em **Rust**, com motor **Chromium** via
**CEF** (Chromium Embedded Framework). Auditoria feita no código, não na documentação:

| Item | Real (verificado) |
|---|---|
| Linguagem / bindings | Rust 2024, crate `cef` **156.4.0+156.0.26** (cef-rs), `cef-dll-sys` |
| Runtime | CEF **156.0.26** (`cef_binary_156.0.26+g8ea0c73+chromium-156.0.8078.88_linuxarm64_minimal`) |
| Chromium | **156.0.8078.88** |
| Interface | **CEF Views** (estilo *Alloy*), desenhada em Rust — **não usa GTK4** (o Terminal é que usa GTK4/VTE) |
| Janela no Linux | X11 (Aura/Ozone do Chromium) gerenciada pelo xfwm4 (XFCE) |
| Versão do app | `Cargo.toml` do workspace = versão da distribuição (`VERSION`, hoje **0.2.1**) |
| Instalação | pacote `osmarcio-browser_<versão>_<arch>.deb` (desde 0.2.1; antes era uma cópia em `/opt`) |

## Módulos (`apps/osmarcio-browser/src/`)

| Arquivo | Papel |
|---|---|
| `main.rs` | modelo multiprocesso (`execute_process`), CLI `--version`, `--check-updates`, `--about`, `--help`; `initialize` → `run_message_loop` → `shutdown` |
| `app.rs` | `App` do CEF, perfil/cache XDG, sandbox ligado |
| `ui.rs` | janela principal, abas, barra, menu ☰, atalhos, **ciclo de vida da janela** ([detalhes](OSMarcioBrowser_Window_Lifecycle.md)) |
| `client.rs` | handlers do CEF: título, carregamento, popups→abas, downloads, certificados, `DoClose`/`OnBeforeClose` |
| `about.rs` | janela **Sobre**, ícone da janela ([Sobre](OSMarcioBrowser_About.md), [Ícones](OSMarcioBrowser_Icons.md)) |
| `version.rs` | **fonte única** das versões (Sobre, `--version`, logs) |
| `updates.rs` | coleta para a verificação de atualizações (dpkg, apt-cache, curl) |
| `crates/osmarcio-core/src/updates.rs` | **decisão** de atualização, pura e com testes unitários ([Atualizações](OSMarcioBrowser_Updates.md)) |
| `assets/icons/` | ícone original (SVG + PNG 16–256) |
| `packaging/` | `.desktop` e conffile `browser-updates.toml` |

Não foi criada a árvore `window/ chromium/ lifecycle/ platform/` sugerida:
o app tem ~1 500 linhas e os módulos acima já separam as responsabilidades
(KISS). O ciclo de vida fica em `ui.rs` + `client.rs`, onde o CEF entrega os eventos.

## Processos

```
osmarcio-browser (browser process, UI Rust) ── zygote ── renderers (sandbox seccomp + PID ns)
                                            ├─ gpu-process / utility / network
```
Todos usam o mesmo executável (`--type=...`). Encerramento correto = nenhum
deles sobrevive após `shutdown()`.

## Comandos (Makefile)

`browser-build` · `browser-icons` · `browser-package` · `browser-version` ·
`browser-check-updates` · `browser-test` (= unitários + `test-browser-hotfix`) ·
`browser-run` (só Linux com sessão gráfica) · `browser-install` (só Debian,
com confirmação) · `browser-release` (copia .deb + SHA-256 para `images/releases/`).

## Plataformas

O código não tem nada específico de arquitetura (o `.deb` usa `dpkg
--print-architecture`; o autoteste compara com `uname -m`). **Testado só em
ARM64** (QEMU/HVF). x86_64 exige o CEF `linux64` e build num host/contêiner
amd64 — **não executado** nesta etapa. ROCK Pi 4A: sem hardware.
