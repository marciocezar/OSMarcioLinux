# Manifesto — OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0.img

| Campo | Valor |
|---|---|
| Distribuição | OSMarcio Linux 0.2.0 |
| Base | Debian 13 (trixie) |
| Arquitetura | arm64 |
| Plataforma | utm |
| Perfil | desktop |
| Kernel | 6.12.116+deb13-arm64 |
| Bootloader | GRUB 2.12-9+deb13u2 (arm64-efi, caminho removível EFI/BOOT/BOOTAA64.EFI) |
| systemd | 257.13-1~deb13u1 |
| Pacotes | 434 (lista: `OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0.img.packages.tsv`) |
| Data de build (UTC) | 2026-11-13T11:33:33Z |
| Commit | 8057a6e (árvore com alterações não commitadas) |
| SHA-256 | `8780d9b0a87dd50bbc3a39c2c017f1a17c11e393c70d77ddd954dce67faa3e62` |
| Tamanho | 1.5G em disco (sparse), 2825 MiB lógicos |

## Resultado dos testes
- 20261113T113805Z: boot QEMU/HVF (UEFI, 2 boots) — **APROVADO** — `reports/tests/boot-arm64-20261113T113805Z.md`

## Limitações
Veja `PROGRESS.md` e `reports/OSMarcioLinux_Platform_Compatibility.md`.

## Instruções
Veja `docs/utm/UTM_Guide.md` (ARM64) ou `docs/virtualbox/VirtualBox_Guide.md` (x86_64).
