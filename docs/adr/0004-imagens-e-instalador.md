# ADR 0004 — Construção de imagens e estratégia de instalação

- **Status:** Aceita para ARM64/UTM; proposta para x86_64 e RK3399

## Ferramentas avaliadas

| Ferramenta | Uso | Decisão |
|---|---|---|
| **mmdebstrap** | gera o rootfs Debian em um `.tar` | ✅ adotado (rápido: ~50 s; não exige VM; hooks simples) |
| debootstrap | idem, mais antigo | descartado (mais lento, sem saída em tar) |
| **live-build** | ISO live híbrida | candidato para a ISO x86_64 (Fase 5) |
| debian-installer | instalador oficial em modo texto/GTK | descartado como UI principal (pouco customizável) |
| **Calamares** | instalador gráfico usado por várias distros | candidato para a Fase 15 (precisa do perfil desktop) |
| systemd-repart | particiona/cresce discos | alternativa futura ao `growpart` |

## Decisão para ARM64 (UTM) — implementada

Imagem **RAW/GPT pronta para uso** (não um instalador):

```
disco.img (GPT)
├── p1  ESP  FAT32  256 MiB   EFI/BOOT/BOOTAA64.EFI  (GRUB standalone)
└── p2  raiz ext4   ~1,3 GiB  Debian + OSMarcio      (cresce no 1º boot)
```

- Montada **sem loop devices nem mount**: `mkfs.vfat` + `mtools` para a ESP,
  `mkfs.ext4 -d` para a raiz, `sgdisk` + `dd` para juntar. Portanto o script
  só manipula arquivos — não há como apontar para um disco físico.
- GRUB no **caminho removível** `EFI/BOOT/BOOTAA64.EFI`: o firmware o acha
  sem variáveis NVRAM (UTM/QEMU podem descartar a NVRAM).
- **Primeiro boot** (`osmarcio-firstboot.service`): expande a partição raiz
  até o fim do disco virtual, gera `machine-id`, cria o usuário `osmarcio`
  com **senha aleatória e troca obrigatória** — nenhuma senha fixa na imagem.
- O tipo de partição raiz é o "Linux root (ARM64)" das *Discoverable
  Partitions* (GUID B921B045…), o que habilita autodetecção pelo systemd.

Para UTM isso é melhor que ISO + instalador: importar um disco já pronto é
o fluxo natural de VMs ARM64, e a ISO x86_64 não funciona no UTM ARM64.

## Proposta para x86_64 (VirtualBox) — Fases 5 e 15

ISO híbrida UEFI com ambiente live (live-build) e instalador (Calamares),
nome `OSMarcioLinux-x86_64-Desktop-v<versão>.iso`. O mesmo
`mkimage-uefi.sh` já aceita `amd64` (gera `BOOTX64.EFI`), o que permite um
marco intermediário: uma imagem de disco x86_64 convertida para VDI.

## Proposta para ROCK Pi 4A — Fase 14

Imagem própria, **nunca derivada da imagem UTM**: U-Boot (com TF-A/BL31 do
RK3399) gravado nos setores reservados do microSD (idbloader em 32 KiB,
u-boot.itb em 8 MiB), `extlinux.conf` ou EFI via U-Boot, DTB
`rockchip/rk3399-rock-pi-4a.dtb`. Sem alterar a SPI flash da placa.
