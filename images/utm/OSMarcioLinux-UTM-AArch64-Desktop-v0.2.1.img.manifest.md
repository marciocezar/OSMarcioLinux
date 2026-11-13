# Manifesto — OSMarcioLinux-UTM-AArch64-Desktop-v0.2.1.img

| Campo | Valor |
|---|---|
| Distribuição | OSMarcio Linux 0.2.1 |
| Base | Debian 13 (trixie) |
| Arquitetura | arm64 |
| Plataforma | utm |
| Perfil | desktop |
| Kernel | 6.12.116+deb13-arm64 |
| Bootloader | GRUB 2.12-9+deb13u2 (arm64-efi, caminho removível EFI/BOOT/BOOTAA64.EFI) |
| systemd | 257.13-1~deb13u1 |
| Pacotes | 435 (lista: `OSMarcioLinux-UTM-AArch64-Desktop-v0.2.1.img.packages.tsv`) |
| Data de build (UTC) | 2026-11-13T12:53:08Z |
| Commit | 2a50729 (árvore com alterações não commitadas) |
| SHA-256 | `eaeaf482e2906d88ea845d9a3e6d1b1706c42b86c354b3a697dc8a6799f8071d` |
| Tamanho | 1.5G em disco (sparse), 2826 MiB lógicos |

## Resultado dos testes
- 20261113T125308Z: boot QEMU/HVF (UEFI, 2 boots) — **APROVADO** — `reports/tests/boot-arm64-20261113T125308Z.md`

## Limitações
Veja `PROGRESS.md` e `reports/OSMarcioLinux_Platform_Compatibility.md`.

## Instruções
Veja `docs/utm/UTM_Guide.md` (ARM64) ou `docs/virtualbox/VirtualBox_Guide.md` (x86_64).
