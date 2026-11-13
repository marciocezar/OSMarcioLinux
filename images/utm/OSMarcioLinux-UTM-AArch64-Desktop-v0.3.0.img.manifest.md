# Manifesto — OSMarcioLinux-UTM-AArch64-Desktop-v0.3.0.img

| Campo | Valor |
|---|---|
| Distribuição | OSMarcio Linux 0.3.0 |
| Base | Debian 13 (trixie) |
| Arquitetura | arm64 |
| Plataforma | utm |
| Perfil | desktop |
| Kernel | 6.12.116+deb13-arm64 |
| Bootloader | GRUB 2.12-9+deb13u2 (arm64-efi, caminho removível EFI/BOOT/BOOTAA64.EFI) |
| systemd | 257.13-1~deb13u1 |
| Pacotes | 453 (lista: `OSMarcioLinux-UTM-AArch64-Desktop-v0.3.0.img.packages.tsv`) |
| Data de build (UTC) | 2026-11-13T15:13:26Z |
| Commit | f0e72c1 (árvore com alterações não commitadas) |
| SHA-256 | `d39cb6725a6f6b494351e4185afb09be29872491e81eb73d8a5edeea053419ce` |
| Tamanho | 1.5G em disco (sparse), 2853 MiB lógicos |

## Resultado dos testes
- 20261113T151326Z: boot QEMU/HVF (UEFI, 2 boots) — **APROVADO** — `reports/tests/boot-arm64-20261113T151326Z.md`

## Limitações
Veja `PROGRESS.md` e `reports/OSMarcioLinux_Platform_Compatibility.md`.

## Instruções
Veja `docs/utm/UTM_Guide.md` (ARM64) ou `docs/virtualbox/VirtualBox_Guide.md` (x86_64).
