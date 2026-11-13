# OSMarcio Linux — Auditoria para a evolução do desktop (0.2.1 → 0.3.0)

Data: 2026-11-13 · ambiente: macOS Apple Silicon (`/Users/marcio/_learn/OSMarcioLinux`).
Git: branch `main`, commits do usuário `8057a6e first commit` e `2a50729 Changes`;
alterações desta etapa não commitadas. O projeto antigo `/Users/marcio/_learn/OSMarcio`
não foi tocado.

## Estado real encontrado (verificado no código e na VM)
| Componente | Implementação real | Estado antes |
|---|---|---|
| Distribuição | Debian 13 + Linux 6.12 LTS, `VERSION` 0.2.1 | imagem Desktop ARM64 testada (QEMU + app UTM) |
| Sessão gráfica | LightDM → Xorg (modesetting, virtio-gpu) → XFCE 4.20, xfwm4 | ok |
| Terminal | Rust + **GTK4 + VTE** (`apps/osmarcio-terminal`), `GtkNotebook` | abas sem largura mínima (`max_width_chars(24)` apenas) |
| Browser | Rust + **CEF Views** (cef-rs 156.4.0, Chromium 156) — **não usa GTK** | não maximizava (ver abaixo) |
| Info | CLI Rust | ok |
| Gerenciador de arquivos | inexistente (`files-build` era [planejado]; entrada do XFCE oculta) | — |
| Campo Minado | inexistente | — |
| Pacotes | `.deb` de terminal, info, browser (deb.sh / browser-deb.sh) | sem ícones genéricos no deb.sh |
| Testes | unitários (core 30, terminal 3), GUI Xvfb do terminal, VM: boot, desktop, hotfix do browser | — |
| Plataformas | ARM64 (QEMU/HVF, app UTM) | x86_64 e ROCK Pi nunca construídos |

## Problemas e causas (medidos)
1. **Abas do Terminal pequenas:** rótulo sem `width_chars`; GTK encolhe até "…".
2. **Browser não maximiza:** `xprop` na VM: `WM_NORMAL_HINTS min = max = 1100×760`
   e `_NET_WM_ALLOWED_ACTIONS` sem MAXIMIZE/RESIZE. Causa no cef-rs:
   `can_resize/can_maximize/can_minimize` retornam 0 por padrão e, sem
   `minimum/maximum_size`, o CEF Views usa o tamanho preferido como limites.

## Arquivos afetados
Terminal: `apps/osmarcio-terminal/src/window.rs`, `.desktop`, ícone.
Browser: `apps/osmarcio-browser/src/{ui,about}.rs`.
Novos: `apps/osmarcio-files/**`, `apps/osmarcio-minesweeper/**`.
Integração: `distro/overlays/desktop/**` (mimeapps, helpers, painel, xsettings),
`distro/profiles/desktop/packages.list`, `scripts/build/{deb.sh,app-icons.sh,customize-rootfs.sh}`,
selftests, `Makefile`, testes em `scripts/test/`.

## Versão
Política (CHANGELOG): SemVer da distribuição. Dois aplicativos novos →
**minor: 0.3.0** (não a 0.2.0 sugerida, que já existia). Mantida em `VERSION`,
`Cargo.toml` (todos os crates/pacotes), `/etc/os-release`, `osmarcio-info`,
Sobre do Browser, manifestos e nome das imagens.

## Testes planejados
Ver `docs/testing/Desktop_Application_Test_Plan.md`.
