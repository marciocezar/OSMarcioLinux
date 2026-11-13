# OSMarcio Linux 0.3.0 — relatório de regressão do desktop

- Data: 2026-11-13 · Host: macOS Apple Silicon · QEMU 11.0.1 + HVF · contêiner Debian 13 arm64
- Imagem: `images/utm/OSMarcioLinux-UTM-AArch64-Desktop-v0.3.0.img` (SHA-256 `d39cb6725a6f6b49…`, `make verify-images` OK)
- Pacotes: `osmarcio-{terminal,info,browser,files,minesweeper}_0.3.0_arm64.deb`

## Resultado final (VMs novas, rodada 15:13–15:20 UTC)
| Teste | Resultado | Evidência |
|---|---|---|
| `make apps-lint` (fmt + clippy -D warnings, workspace) | OK | — |
| `make apps-test` (unitários + GUI do Terminal em Xvfb) | core 30 · Files 15 · Campo Minado 18 · Terminal 3 + GUI 10 — **0 falhas** | — |
| `make test-boot` (2 boots: rede, DNS, HTTPS, APT, persistência, LightDM, 5 apps, ícones distintos, MIME) | **0 falhas** | `reports/tests/boot-arm64-20261113T151326Z.md` |
| `make test-desktop-apps` (login real; Terminal, Browser, Files, Campo Minado) | **51 PASS, 0 FAIL** | `reports/tests/desktop-apps-arm64-20261113T151401Z.md` + 11 capturas |
| `make test-browser-hotfix` (fechamento, ícone, Sobre, atualizações — regressão 0.2.1) | **35 PASS, 0 FAIL** | `reports/tests/browser-hotfix-arm64-20261113T151758Z.md` |
| **App UTM 4.7.5** (VM "OSMarcio Linux 0.3.0 Desktop" via `make run-utm`): login real + todas as fases de `test-desktop-apps` | **51 PASS, 0 FAIL**; Browser maximizado com renderização até a borda (107/18 cores, iguais ao QEMU) | `reports/tests/utm-app-030-20261113T1530Z.log` + 11 capturas `utm-app-030-*` |

Regressão coberta: boot, login, desktop, Terminal, Browser (HTTPS, abas, fechamento,
CEF), rede, APT, sistema de arquivos, persistência entre boots, desligamento
(poweroff do autoteste) e reinício (2º boot).

## Histórico honesto desta rodada
As primeiras execuções do teste gráfico falharam e revelaram defeitos reais, todos
corrigidos e re-testados: navegação por setas e cores do Campo Minado; no Files,
progresso com links, seleção do nome ao renomear, foco na barra de endereço,
diálogo modal ao cancelar e deslocamento da lista na lixeira; no Browser, a métrica
de renderização do teste estava errada (a página estava correta). Imagens e pacotes
intermediários foram movidos (não apagados) para `out/superseded/*.build1…5`.

## Plataformas
| Plataforma | Estado |
|---|---|
| Linux ARM64 — QEMU AArch64 (HVF) | Implementado e testado |
| Linux ARM64 — app UTM 4.7.5 | Implementado e testado (51/51; ver abaixo) |
| Linux x86_64 — QEMU/VirtualBox | **Bloqueado**: não há build amd64 nem ISO; o código não depende de arquitetura |
| ROCK Pi 4A | **Bloqueado**: sem hardware |
| WSL2 (build) | não executado nesta etapa |

## Teste no app UTM
Execução dentro da VM pelo qemu-guest-agent (`utmctl exec/file`), com entrada
XTest e capturas do display X da própria VM (o macOS não concede gravação de
tela a este agente). Sistema `running`, 1280x800, os 5 pacotes 0.3.0. O menu
mostra as categorias Internet, Jogos e Sistema com os ícones próprios. Depois
do teste a VM foi devolvida ao **1º boot** (nova senha temporária pelo
`firstboot.sh`, ImageMagick removido, dados de teste apagados,
`dpkg --verify` sem alterações nos 5 pacotes) e ficou ligada na tela de login.
Não verificado: teclado/mouse físicos do Mac na janela do UTM.

## Pendências
Build x86_64 (CEF linux64, ISO, VirtualBox); escala 2× e
tema alternativo; clique do mouse no tabuleiro do Campo Minado automatizado;
gvfs para Recentes/montagem no Files.
