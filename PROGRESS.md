# PROGRESS — OSMarcio Linux 0.1.0

Estados: **Implementado e testado** · **Implementado sem teste completo** ·
**Parcial** · **Bloqueado** · **Planejado**. Atualizado em 2026-11-13.

| Fase | Estado |
|---|---|
| 1 Assessment | Implementado e testado |
| 2 Workspace e Setup | Parcial (Mac testado; Windows/WSL2 não executados) |
| 3 Linux Minimal | Implementado e testado (ARM64) |
| 4 UTM ARM64 | Parcial (QEMU/HVF testado; app UTM não) |
| 5 VirtualBox x86_64 | Planejado |
| 6 Desktop | Planejado (ADR 0003 proposta) |
| 7 Terminal Rust | Implementado sem teste completo |
| 8 Editor | Planejado |
| 9 File Manager | Planejado |
| 10 Browser + CEF | Planejado (validação documental feita; POC pendente) |
| 11 Software Center | Planejado |
| 12 Aplicativos externos | Planejado |
| 13 Settings | Planejado |
| 14 ROCK Pi 4A | Bloqueado (requer hardware e BSP) |
| 15 Instalador | Planejado |
| 16 Testes | Parcial |
| 17 Documentação | Parcial |
| 18 Release | Planejado |

---

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

## Fase 4 — UTM ARM64 · Parcial
- **Implementado:** IMG RAW/GPT/UEFI, GRUB em caminho removível, expansão da raiz, manifesto + SHA-256, `make run-qemu-arm64`, `make test-utm`, guia UTM.
- **Testes:** 2 boots no QEMU 11 + HVF, 0 falhas (boot, login, rede, DNS, HTTPS, NTP, APT install/remove, persistência).
- **Não testado:** importação e boot no **aplicativo** UTM; display gráfico (só console).
- **Próximo passo:** seguir `docs/utm/UTM_Guide.md` e registrar evidência.

## Fase 5 — VirtualBox x86_64 · Planejado
- `mkimage-uefi.sh` e `rootfs.sh` já aceitam `amd64`, sem execução. Script de VM escrito, não testado. Nenhuma ISO.

## Fase 7 — Terminal Rust · Implementado sem teste completo
- **Implementado:** GTK4 + VTE, abas (nova, fechar, fechar outras, renomear, reordenar, Ctrl+T/W/Tab), regra da última aba, copiar/colar, zoom, temas, configuração XDG, `.deb`.
- **Testes:** 12 unitários; 10/10 de GUI em Xvfb; `.deb` instalado/removido em Debian 13 limpo.
- **Falta:** testar dentro do OSMarcio Linux com desktop real (Wayland), janela de preferências, ícone próprio, aba nova no diretório atual.
- **Arquivos:** `apps/osmarcio-terminal/`, `crates/osmarcio-core/`, `docs/terminal/OSMarcio_Terminal.md`.

## Fase 10 — Browser · Planejado
- Validação documental (versão, arquiteturas declaradas, licença) na ADR 0005. Nenhum código: regra "não construir o Browser antes de validar CEF".

## Fase 14 — ROCK Pi 4A · Bloqueado
- Sem hardware nesta sessão. Nenhuma imagem será criada a partir da imagem UTM.

## Fase 16 — Testes · Parcial
- Unitários, lint, GUI do terminal, pacote, boot integrado ARM64. Faltam: x86_64, instalação, desktop, browser.

## Fase 17 — Documentação · Parcial
- Fundamentos, boot, rede, terminal, UTM, VirtualBox (planejado), Makefile, licenças, ADRs, 4 relatórios, diagrama draw.io com 7 abas + PNG.
- Faltam: 20 dos 27 diagramas pedidos (os demais descrevem componentes que ainda não existem), relatórios de Browser/Install/Release (serão criados quando houver conteúdo real).

## Limitações conhecidas (globais)
- Nenhum desktop gráfico ainda; a imagem é só console.
- Nenhum commit git foi feito (o manifesto mostra "sem-commit").
- Licença GPL-3.0-or-later escolhida provisoriamente.
