# Relatório de build — 2026-11-13

## Imagem ARM64 (UTM) — perfil Minimal

| Item | Valor |
|---|---|
| Artefato | `images/utm/OSMarcioLinux-UTM-AArch64-Minimal-v0.1.0.img` |
| SHA-256 | `307ba9e1123bd22ae5801583780b4d9727acc1db57e5a32f346a1c1e0096e51c` |
| Tamanho | 1565 MiB lógicos; ~588 MiB reais (arquivo esparso) |
| Kernel | 6.12.116+deb13-arm64 |
| Bootloader | GRUB 2.12-9+deb13u2, `EFI/BOOT/BOOTAA64.EFI` |
| systemd / APT / OpenSSL | 257.13 / 3.0.4 / 3.5.7 |
| Pacotes | 205 (`…img.packages.tsv`) |
| Tempo de build | rootfs ~52 s (mmdebstrap) + imagem ~30 s, no container arm64 nativo |
| Uso medido em boot (2 GiB RAM) | ~160 MiB RAM usada, ~100 processos, 768 MB na raiz |
| Teste | `make test-utm` → **0 falhas**, 22 verificações × 2 boots, ~15 s por ciclo de boot (relógio de parede, inclui `apt update/install`) — `reports/tests/boot-arm64-20261113T023007Z.md` |

Histórico desta sessão (imagens substituídas guardadas em `out/superseded/`):

| Build | Resultado | Motivo da substituição |
|---|---|---|
| 1 | 2 falhas | bug de SIGPIPE no firstboot (usuário sem senha); verificação APT errada |
| 2 | 0 falhas | GRUB com caminho fixo do kernel (quebraria em upgrade de kernel) |
| 3 | 0 falhas | medidas de RAM vazias (locale) |
| 4 (atual) | 0 falhas | — |

## Pacote `.deb`

| Item | Valor |
|---|---|
| Artefato | `out/debs/osmarcio-terminal_0.1.0_arm64.deb` (não versionado no git) |
| Depends | `libgtk-4-1, libvte-2.91-gtk4-0, bash` |
| Teste | em `debian:trixie` limpo: `apt-get install ./…deb` → `Status: install ok installed`; `ldd`: 0 bibliotecas faltando; `apt-get remove` OK |

## Rust

`cargo build`, `clippy -D warnings`, `cargo fmt --check`, 12 testes
unitários e 10 testes de GUI — todos passando no container Debian 13.
