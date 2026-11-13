# Manifesto — OSMarcioLinux-UTM-AArch64-Minimal-v0.1.0.img

| Campo | Valor |
|---|---|
| Distribuição | OSMarcio Linux 0.1.0 |
| Base | Debian 13 (trixie) |
| Arquitetura | arm64 |
| Plataforma | utm |
| Perfil | minimal |
| Kernel | 6.12.116+deb13-arm64 |
| Bootloader | GRUB 2.12-9+deb13u2 (arm64-efi, caminho removível EFI/BOOT/BOOTAA64.EFI) |
| systemd | 257.13-1~deb13u1 |
| Pacotes | 205 (lista: `OSMarcioLinux-UTM-AArch64-Minimal-v0.1.0.img.packages.tsv`) |
| Data de build (UTC) | 2026-11-13T02:30:07Z |
| Commit | sem-commit (árvore com alterações não commitadas) |
| SHA-256 | `307ba9e1123bd22ae5801583780b4d9727acc1db57e5a32f346a1c1e0096e51c` |
| Tamanho | 588M em disco (sparse), 1565 MiB lógicos |

## Resultado dos testes
- 20261113T023007Z: boot QEMU/HVF (UEFI, 2 boots) — **APROVADO** — `reports/tests/boot-arm64-20261113T023007Z.md`

## Limitações
Veja `PROGRESS.md` e `reports/OSMarcioLinux_Platform_Compatibility.md`.

## Instruções
Veja `docs/utm/UTM_Guide.md` (ARM64) ou `docs/virtualbox/VirtualBox_Guide.md` (x86_64).
