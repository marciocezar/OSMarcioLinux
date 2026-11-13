# Manifesto — OSMarcioLinux-UTM-AArch64-Minimal-v0.2.0.img

| Campo | Valor |
|---|---|
| Distribuição | OSMarcio Linux 0.2.0 |
| Base | Debian 13 (trixie) |
| Arquitetura | arm64 |
| Plataforma | utm |
| Perfil | minimal |
| Kernel | 6.12.116+deb13-arm64 |
| Bootloader | GRUB 2.12-9+deb13u2 (arm64-efi, caminho removível EFI/BOOT/BOOTAA64.EFI) |
| systemd | 257.13-1~deb13u1 |
| Pacotes | 205 (lista: `OSMarcioLinux-UTM-AArch64-Minimal-v0.2.0.img.packages.tsv`) |
| Data de build (UTC) | 2026-11-13T10:52:28Z |
| Commit | sem-commit (árvore com alterações não commitadas) |
| SHA-256 | `7d424f469c7be9afd1ba3652327255ae812fa8ceefcd5a969c0a276a10c96040` |
| Tamanho | 588M em disco (sparse), 1565 MiB lógicos |

## Resultado dos testes
Ainda não testado. Rode `make test-utm` (ou o alvo de teste da plataforma);
o relatório vai para `reports/tests/`.

## Limitações
Veja `PROGRESS.md` e `reports/OSMarcioLinux_Platform_Compatibility.md`.

## Instruções
Veja `docs/utm/UTM_Guide.md` (ARM64) ou `docs/virtualbox/VirtualBox_Guide.md` (x86_64).
