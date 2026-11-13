# OSMarcio Browser — Relatório do hotfix 0.2.1

- Data: 2026-11-13 · Host: macOS Apple Silicon · QEMU 11.0.1 + HVF · contêiner Debian 13 arm64
- Versão: distribuição e Browser **0.2.0 → 0.2.1** (SemVer *patch*, conforme o CHANGELOG; a imagem 0.2.0 foi preservada)
- Motor: **Chromium 156.0.8078.88**, **CEF 156.0.26**, bindings **cef-rs 156.4.0+156.0.26** (inalterados)

## Auditoria inicial (no código)
A interface é **CEF Views em Rust (estilo Alloy)** — não GTK4 (GTK4/VTE é o
Terminal). Janela X11 do Chromium gerenciada pelo xfwm4. Ciclo de vida em
`src/ui.rs` (`WindowDelegate`) e `src/client.rs` (`LifeSpanHandler`). Ícone:
o padrão do Chromium (128×128). `.desktop` com `Icon=web-browser`. Instalação:
cópia do bundle em `/opt/osmarcio/browser` (sem pacote). Atualização: nenhuma.
Detalhes: [docs/browser/OSMarcioBrowser_Architecture.md](../docs/browser/OSMarcioBrowser_Architecture.md).

## 1. Causa raiz do botão X sem função
`can_close` chamava `close_browser(0)` em cada aba e **sempre retornava 0**.
Com CEF Views, o browser de uma `BrowserView` só termina de fechar quando a
janela fecha; o `DoClose` padrão pede de novo o fechamento da janela e
`can_close` respondia 0 outra vez — impasse. Reproduzido na VM: clique real no
✕ (e Alt+F4, Ctrl+Q) → janela aberta, 9 processos vivos, log parado em
`atalho cmd=15`. Ao corrigir, apareceu um 2º defeito: **Ctrl+W com várias abas
fechava a janela inteira** (mesmo `DoClose` padrão).

## 2. Arquivos modificados
| Novos | Alterados |
|---|---|
| `apps/osmarcio-browser/src/{about,version,updates}.rs` | `apps/osmarcio-browser/src/{ui,client,main}.rs`, `build.rs` |
| `apps/osmarcio-browser/assets/icons/**` (SVG + 7 PNG + scalable) | `crates/osmarcio-core/src/{lib,web}.rs` (`base64` público) |
| `apps/osmarcio-browser/packaging/browser-updates.toml` | `.desktop` movido para `apps/osmarcio-browser/packaging/` (Icon, ação Sobre) |
| `crates/osmarcio-core/src/updates.rs` (+10 testes) | `Makefile`, `VERSION`, `Cargo.toml/.lock`, `.gitignore` (`*.deb`) |
| `scripts/build/{browser-deb,browser-icons}.sh`, `scripts/release/browser-release.sh` | `scripts/build/customize-rootfs.sh` (instala o `.deb`), `scripts/run/qemu-arm64.sh` (`--qga`) |
| `scripts/test/{qga.py,test-browser-hotfix.sh,test-browser-hotfix-qemu.sh}` | `scripts/test/test-desktop-qemu-arm64.sh`, selftests (caminho `/usr/lib/osmarcio-browser`, arquitetura genérica, checagens do pacote) |
| `docs/browser/*.md` (6) | overlay desktop: lançador/helper com o ícone, `xsettings.xml` (ícones nos menus), removido o wrapper `/usr/bin` |
| | `README.md`, `PROGRESS.md`, `CHANGELOG.md` · `cargo fmt` reformatou `osmarcio-info`/`sysinfo.rs` (só formatação) |

## 3–4. Solução e ciclo de vida do CEF
- `can_close`: `try_close_browser()` em todas as abas; retorna 1 só quando todas
  podem fechar (o CEF reenvia o pedido até lá). Fecha o Sobre junto.
- `do_close`: janela fechando → 0; **várias abas** → retira a aba da interface,
  solta as referências à `BrowserView` (o CEF destrói o browser) e retorna 1;
  **última aba** → marca fechamento e retorna 0 (mesmo caminho do X).
- `on_window_destroyed` + `on_before_close` → `maybe_quit()`: só chama
  `quit_message_loop()` com janela destruída, 0 abas e Sobre fechado; então
  `shutdown()` roda sem browsers vivos. Id do browser guardado na aba.
- Nenhum `kill` ou encerramento forçado.
Detalhes: [OSMarcioBrowser_Window_Lifecycle.md](../docs/browser/OSMarcioBrowser_Window_Lifecycle.md).

## 5–7. Ícone e integração
Ícone original (globo + órbita com seta, cores OSMarcio), sem marcas de
terceiros; PNG RGBA 16/24/32/48/64/128/256 + SVG, gerados de um único SVG.
Aplicado com `set_window_icon`/`set_window_app_icon` (`_NET_WM_ICON` 16 e 128),
`.desktop` (`Icon=osmarcio-browser`), lançador do painel, tela Sobre e pacote
(`/usr/share/icons/hicolor`). Ligado também `MenuImages` no XFCE (o menu não
mostrava ícone nenhum). Limitação: **WM_CLASS** pedido ao CEF não é aplicado
(ver [Icons](../docs/browser/OSMarcioBrowser_Icons.md)).

## 8–9. Tela Sobre e versões
Menu ☰ → "Sobre o OSMarcio Browser", F1 ou `--about`. Versões lidas do
**libcef.so carregado** (`cef_version_info`, FFI declarado em `version.rs`) e
comparadas com a versão compilada (`runtime_confere=true`). Mesma fonte para
`--version` e logs.

| Campo | Valor medido na imagem 0.2.1 |
|---|---|
| OSMarcio Browser | 0.2.1 |
| Chromium Engine | 156.0.8078.88 |
| CEF Runtime / compilado | 156.0.26 / 156.0.26+g8ea0c73+chromium-156.0.8078.88 |
| cef-rs | 156.4.0+156.0.26 |
| Arquitetura / Sistema | aarch64 / OSMarcio Linux 0.2.1 |

## 10. Estratégia de atualização
Não existe repositório APT assinado nem servidor de releases → o app **verifica
e informa, não instala** (pedido do item 7.2). Ordem: pacote instalado mais novo
que o processo (reinício necessário) → candidato de **repositório** APT →
manifesto homologado (`https://`/`file:///`, TLS verificado, exige
`homologated` + `sha256` + mesma arch). Instalação só via `sudo apt`.
Detalhes: [OSMarcioBrowser_Updates.md](../docs/browser/OSMarcioBrowser_Updates.md).

## 11–12. Testes e resultados

| Teste | Resultado | Evidência |
|---|---|---|
| `cargo clippy --workspace -D warnings`, `cargo fmt --check` | OK (inclui os 2 avisos antigos, corrigidos) | — |
| `cargo test -p osmarcio-core` | 30 OK (10 novos de atualização) | — |
| `make test-boot` (imagem 0.2.1, 2 boots) | 0 falhas; pacote, arch, libs, `--version`, 8 ícones | `reports/tests/boot-arm64-20261113T125308Z.md` |
| `make test-desktop` (fase 0) | 0 falhas; página renderizada | `reports/tests/desktop-arm64-20261113T125344Z.md` |
| `make test-browser-hotfix` | **32 PASS, 0 FAIL** (+1 limitação WM_CLASS); após corrigir a limpeza do script: **35 PASS** (inclui conffile, IPv6 e rede restaurados) | `reports/tests/browser-hotfix-arm64-20261113T125344Z.md` (+4 capturas), `…131702Z.md` |
| Reinício necessário | processo 0.2.0 + pacote 0.2.1 instalado → "Reinício necessário" | VM de desenvolvimento (manual) |
| **App UTM 4.7.5** (VM "OSMarcio Linux 0.2.1 Desktop", `make run-utm`) | **33 PASS, 0 FAIL**: login gráfico + as mesmas fases (fechamento, Sobre, atualizações) | `reports/tests/utm-app-browser-hotfix-20261113T1310Z.log` + 5 capturas `utm-app-browser-hotfix-20261113T1310Z-*.png` |
| Alvos `browser-*` do Makefile | version/check-updates/icons/package/release OK; run/install terminam com erro 2 explicativo no macOS | — |

Cenários cobertos no `test-browser-hotfix`: X com 1 aba · Alt+F4 com 3 abas ·
Ctrl+Q com página carregando · X com download ativo · menu ☰ → Fechar janela ·
Ctrl+W com 2 abas (só a aba) · Ctrl+W na última aba · reabrir · ícone na
janela · Sobre (abrir, versões, Verificar, Copiar, Fechar) · X com Sobre aberto ·
atualização: fonte ausente, já atualizado, nova homologada, não homologada,
incompatível, `http://` recusado, DNS, TLS, resposta inválida, sem internet.
Todos os fechamentos terminam com `osmb: encerrado` e **0 subprocessos**.

Defeitos achados pelos próprios testes durante o hotfix e corrigidos: Ctrl+W
fechando a janela; candidato do `/var/lib/dpkg/status` tratado como repositório
(dizia "atualizado" sem fonte); teste "sem internet" que só removia a rota IPv4.
Os builds 0.2.1 com esse defeito foram movidos para `out/superseded/*.build1`.

Artefatos: `images/utm/OSMarcioLinux-UTM-AArch64-Desktop-v0.2.1.img`
(SHA-256 `eaeaf482…8071d`), `images/releases/osmarcio-browser_0.2.1_arm64.deb`
(SHA-256 `820d0ceb…dba51`).

## 13. Plataformas verificadas
| Plataforma | Estado |
|---|---|
| Linux ARM64 — QEMU AArch64 (HVF) | **Implementado e testado** |
| Linux ARM64 — app UTM 4.7.5 | **Implementado e testado** (33/33; ver abaixo) |
| Linux x86_64 — QEMU AMD64 / VirtualBox | **Bloqueado**: não há build x86_64 da distribuição nem CEF linux64 baixado; código sem dependência de arquitetura |
| ROCK Pi 4A | **Bloqueado** (sem hardware) |

### Teste no app UTM (Apple Silicon)
VM criada por `make run-utm` (aarch64, HVF, UEFI, disco VirtIO, virtio-gpu-pci,
rede compartilhada). Boot até `running`, LightDM, 1280x800, pacote
`osmarcio-browser 0.2.1`. Como o macOS não dá a este agente permissão de
gravação de tela, o teste foi executado **dentro da VM** pelo
qemu-guest-agent (`utmctl exec/file`): login no LightDM por entrada XTest
(não pelo teclado do Mac), troca obrigatória de senha e `test-browser-hotfix.sh`;
capturas feitas no display X da própria VM.

Observações do UTM:
- A mensagem de estado no Sobre aparece **truncada com "…"** quando é longa
  (ex.: "Fonte de atualização não configurada (nem repositório APT…"). O texto
  completo está no log e em `--check-updates`. Pendência de interface.
- O erro de manifesto inválido traz a mensagem multilinha do parser TOML (só
  na CLI/log; no Sobre fica truncada). Cosmético.
- O **script de teste** deixou dois efeitos colaterais na VM, corrigidos à mão
  e depois no script: o conffile `browser-updates.toml` ficou alterado (a fase
  `sobre-verificar` não guardava o original) e a rota IPv6 padrão não foi
  recriada (o campo `expires` da rota não é aceito pelo `ip route add`). O
  script agora restaura os dois e confere com `dpkg --verify` e HTTPS.
- Depois do teste a VM foi devolvida ao **1º boot**: sessão encerrada, nova
  senha temporária gerada pelo `firstboot.sh`, ImageMagick (usado só para as
  capturas) removido, perfil/downloads de teste apagados, `dpkg --verify` limpo.

## Classificação
| Item | Estado |
|---|---|
| Botão X / Alt+F4 / Ctrl+Q / menu / Ctrl+W | Implementado e testado |
| Ausência de processos órfãos | Implementado e testado |
| Ícone próprio (janela, barra, Alt+Tab, menu, pacote, Sobre) | Implementado e testado |
| WM_CLASS da janela | Parcial (limitação do CEF/cef-rs) |
| Tela Sobre com versões reais | Implementado e testado |
| Verificação de atualizações (estados) | Implementado e testado |
| Instalação de atualização com autorização / pós-atualização | Bloqueado (sem canal APT assinado) |
| Pacote Debian do Browser | Implementado e testado (instalação limpa na imagem e sobre sistema em uso) |
| Múltiplas janelas no mesmo processo | Planejado (o app tem 1 janela por processo; processos independentes não interferem) |

## 14. Pendências
- Repositório APT assinado (chave do projeto, `Release`/`InRelease`) para habilitar a instalação.
- Investigar o WM_CLASS no cef-rs (cópia de volta de `CefString` em `LinuxWindowProperties`).
- Build e teste x86_64.
- Quebra de linha da mensagem de estado no Sobre; resumir o erro do parser TOML numa linha.
- `test-browser-gui.sh` antigo (contêiner) ainda não verifica pixels.

## 15. Próximas melhorias
Repositório APT + manifesto publicado; ícone de "atualização disponível" no menu;
múltiplas janelas no mesmo processo; quebra de linha da mensagem de estado no Sobre.
