# PROGRESS — OSMarcio Linux 0.3.0

Estados: **Implementado e testado** · **Implementado sem teste completo** ·
**Parcial** · **Bloqueado** · **Planejado**. Atualizado em 2026-11-13.

| Fase | Estado |
|---|---|
| 1 Assessment | Implementado e testado |
| 2 Workspace e Setup | Parcial (Mac testado; Windows/WSL2 não executados) |
| 3 Linux Minimal | Implementado e testado (ARM64) |
| 4 UTM ARM64 | Implementado e testado (QEMU/HVF + app UTM 4.7.5) |
| 5 VirtualBox x86_64 | Planejado |
| 6 Desktop | Parcial (XFCE/X11 + LightDM funcionando; sem tema/app "Sobre" próprios) |
| 7 Terminal Rust | Implementado e testado (dentro do desktop OSMarcio, QEMU e UTM) |
| 8 Editor | Planejado |
| 9 File Manager | Implementado e testado (ARM64) |
| 10 Browser + CEF | Parcial (ARM64 testado: fechamento, ícone, Sobre, verificação de atualizações; sem canal APT assinado; x86_64 não construído) |
| 11 Software Center | Planejado |
| 12 Aplicativos externos | Planejado |
| 13 Settings | Planejado |
| 14 ROCK Pi 4A | Bloqueado (requer hardware e BSP) |
| 15 Instalador | Planejado |
| 16 Testes | Parcial |
| 17 Documentação | Parcial |
| 18 Release | Planejado |

---

## Evolução 0.3.0 (2026-11-13) · Implementado e testado (ARM64)
- **Terminal (abas maiores):** Implementado e testado — Xvfb 10/10; VM 10/10.
- **Browser (maximizar):** Implementado e testado — VM 10/10 (cef-rs devolvia 0 em can_resize/can_maximize); regressão do hotfix 0.2.1: 35/35.
- **OSMarcio Files (novo, Fase 9):** Implementado e testado — 15 unitários; VM 20/20.
- **Campo Minado (novo):** Implementado e testado — 18 unitários; VM 10/10.
- **Integração:** ícones próprios dos 4 apps, .desktop, MIME, painel, 5 pacotes .deb; `make test-boot` 0 falhas; `make test-desktop-apps` 51/51.
- **Evidências:** `reports/OSMarcioLinux_Desktop_Regression_Report.md` e os 5 relatórios da etapa.
- **App UTM 4.7.5:** testado — 51/51 (`reports/tests/utm-app-030-20261113T1530Z.log`).
- **Não testado / bloqueado:** x86_64/VirtualBox; ROCK Pi 4A; escala 2×.

## Hotfix 0.2.1 — OSMarcio Browser (2026-11-13)
- **Implementado e testado (ARM64, QEMU/HVF, sessão XFCE real):** fechamento da janela (X, Alt+F4, Ctrl+Q, menu, Ctrl+W, com download/carregamento/Sobre abertos, sem órfãos); ícone original na janela/barra/Alt+Tab/menu/pacote; janela Sobre com versões reais; verificação de atualizações (10 cenários); pacote `.deb`.
- **Parcial:** WM_CLASS não aplicado pelo CEF (ícone funciona via `_NET_WM_ICON`).
- **Bloqueado:** instalação de atualização com autorização (não há repositório APT assinado).
- **App UTM 4.7.5:** testado (33/33, `reports/tests/utm-app-browser-hotfix-20261113T1310Z.log`).
- **Não testado:** x86_64 (CEF linux64 não baixado/compilado).
- **Evidências:** `reports/OSMarcioBrowser_Fix_Report.md`, `reports/tests/browser-hotfix-arm64-*.md`.

## Correção 2026-11-13 — boot no UTM e desktop · Implementado e testado
- **Problema:** a `.img` Minimal v0.2.0 foi anexada no UTM como "Boot ISO Image" (CD USB) + disco VirtIO vazio → UEFI caía no `Shell>`. A imagem também não tinha desktop.
- **Correção:** `make run-utm` cria a VM com a IMG como disco VirtIO; perfil `desktop` (LightDM + XFCE) com Terminal e Browser; bug de layout do Browser (página invisível) corrigido.
- **Artefato:** `images/utm/OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0.img`.
- **Evidências:** `reports/OSMarcioLinux_UTM_Boot_Fix.md`, `reports/tests/desktop-arm64-20261113T114410Z.md`, `reports/tests/boot-arm64-20261113T113805Z.md`, `reports/tests/utm-app-20261113T1145Z-*`, capturas em `reports/tests/screenshots/`.
- **Guia:** `docs/installation/UTM_Quick_Start.md`.

## Fase 1 — Assessment · Implementado e testado
- **Objetivo:** escolher base, kernel, desktop, validar CEF e ferramentas.
- **Implementado:** ADRs 0001–0007; levantamento do host.
- **Arquivos:** `docs/adr/`, `reports/OSMarcioLinux_Assessment.md`.
- **Evidências:** versões reais coletadas do build (kernel 6.12.116, systemd 257.13, crate `cef` 156.4.0).
- **Problemas:** ADR 0003 (desktop) ainda é proposta; ADR 0005 depende de POC.
- **Próximo passo:** protótipo labwc e POC CEF.

## Fase 2 — Workspace e Setup · Parcial
- **Implementado:** Cargo workspace, `rust-toolchain.toml`, Makefile com 60+ alvos (indisponíveis falham com código 2), container de build, `setup-macos.sh`, `setup-wsl.sh`, `setup-windows.ps1`, `create-virtualbox-vm.ps1`.
- **Testes:** `make doctor-mac` → 12/12 OK; `make help`, alvos indisponíveis, `verify-images`.
- **Problemas:** scripts de Windows/WSL2 **nunca executados** (sem máquina Windows nesta sessão).
- **Próximo passo:** rodar `make doctor-wsl` e `setup-windows.ps1 -Check` no Windows.

## Fase 3 — Linux Minimal · Implementado e testado (ARM64)
- **Implementado:** rootfs Debian 13 via mmdebstrap; overlays (os-release, issue, rede, serviços OSMarcio); root bloqueado; primeiro boot com senha aleatória; systemd-networkd/resolved/timesyncd; APT assinado.
- **Arquivos:** `distro/`, `scripts/build/rootfs.sh`, `scripts/build/customize-rootfs.sh`.
- **Evidências:** `reports/tests/boot-arm64-20261113T023007Z.md` (+ logs seriais).
- **Problemas:** firewall sem regras; x86_64 não construído.

## Fase 4 — UTM ARM64 · Implementado e testado
- **Atualização 2026-11-13:** testado no **app UTM 4.7.5** (VM criada por `make run-utm`; evidências via qemu-guest-agent). Não testado: teclado/mouse físicos e resolução dinâmica na janela do UTM.

### Histórico (0.1.0)
- **Implementado:** IMG RAW/GPT/UEFI, GRUB em caminho removível, expansão da raiz, manifesto + SHA-256, `make run-qemu-arm64`, `make test-utm`, guia UTM.
- **Testes:** 2 boots no QEMU 11 + HVF, 0 falhas (boot, login, rede, DNS, HTTPS, NTP, APT install/remove, persistência).
- **Não testado:** importação e boot no **aplicativo** UTM; display gráfico (só console).
- **Próximo passo:** seguir `docs/utm/UTM_Guide.md` e registrar evidência.

## Fase 5 — VirtualBox x86_64 · Planejado
- `mkimage-uefi.sh` e `rootfs.sh` já aceitam `amd64`, sem execução. Script de VM escrito, não testado. Nenhuma ISO.

## Fase 6 — Desktop · Parcial
- **Implementado:** perfil `desktop` (plano B da ADR 0003: XFCE 4.20 sobre X11, LightDM 1.32, Greybird-dark + elementary-xfce), senha temporária do 1º boot na tela de login (`pam_echo`), painel com lançadores, apps padrão do XFCE apontando para os apps OSMarcio.
- **Testes:** `make test-desktop` (login real no greeter, troca de senha, sessão, xfwm4, painel, xfdesktop, resolução) — 0 falhas; mesmo autoteste no app UTM — 12/12.
- **Falta:** tema/ícones próprios, janela "Sobre", gerenciador de arquivos, cliques nos lançadores testados, resolução dinâmica, fontes CJK.

## Fase 7 — Terminal Rust · Implementado e testado
- **Atualização 2026-11-13:** abre no desktop OSMarcio (QEMU e UTM) e executa comandos digitados no bash.

### Histórico
- **Implementado:** GTK4 + VTE, abas (nova, fechar, fechar outras, renomear, reordenar, Ctrl+T/W/Tab), regra da última aba, copiar/colar, zoom, temas, configuração XDG, `.deb`.
- **Testes:** 12 unitários; 10/10 de GUI em Xvfb; `.deb` instalado/removido em Debian 13 limpo.
- **Falta:** testar dentro do OSMarcio Linux com desktop real (Wayland), janela de preferências, ícone próprio, aba nova no diretório atual.
- **Arquivos:** `apps/osmarcio-terminal/`, `crates/osmarcio-core/`, `docs/terminal/OSMarcio_Terminal.md`.

## Fase 10 — Browser · Parcial
- **Implementado (código já existente em `apps/osmarcio-browser`, CEF 156 / Chromium 156):** janela Rust (CEF Views, estilo Alloy), abas, barra de endereços, favoritos, downloads.
- **Testes 2026-11-13 (dentro do OSMarcio Linux ARM64, QEMU e UTM):** binários aarch64, nenhuma lib ausente, janela abre, HTTPS 200, página renderizada (verificação por pixels), sandbox seccomp + PID namespace, sem `--no-sandbox`, ~740 MiB RSS.
- **Corrigido:** BrowserView com largura 0 (cross-axis START do `BoxLayoutSettings::default()` do cef-rs).
- **Falta:** 2 avisos `clippy::collapsible_if`; `test-browser-gui.sh` sem checagem de pixels; fontes CJK.

## Fase 14 — ROCK Pi 4A · Bloqueado
- Sem hardware nesta sessão. Nenhuma imagem será criada a partir da imagem UTM.

## Fase 16 — Testes · Parcial
- Unitários, lint, GUI do terminal, pacote, boot integrado ARM64. Faltam: x86_64, instalação, desktop, browser.

## Fase 17 — Documentação · Parcial
- Fundamentos, boot, rede, terminal, UTM, VirtualBox (planejado), Makefile, licenças, ADRs, 4 relatórios, diagrama draw.io com 7 abas + PNG.
- Faltam: 20 dos 27 diagramas pedidos (os demais descrevem componentes que ainda não existem), relatórios de Browser/Install/Release (serão criados quando houver conteúdo real).

## Limitações conhecidas (globais)
- Desktop sem aceleração 3D (renderização por software) e sem tema próprio.
- Há um único commit ("first commit"); as alterações desta correção não estão commitadas.
- Licença GPL-3.0-or-later escolhida provisoriamente.
