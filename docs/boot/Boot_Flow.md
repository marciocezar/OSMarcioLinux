# Como o OSMarcio Linux inicializa (ARM64 / UTM)

Cada etapa abaixo foi observada no log serial do teste
(`reports/tests/boot-arm64-*-serial1.log`).

```
Firmware UEFI (edk2)
  │  procura \EFI\BOOT\BOOTAA64.EFI na partição ESP (FAT32)
  ▼
GRUB 2.12 "standalone" (gerado por grub-mkimage)
  │  config embutida: search --fs-uuid <raiz>; configfile /boot/grub/grub.cfg
  ▼
grub.cfg → linux /vmlinuz root=UUID=… ro console=tty0 console=ttyAMA0
          initrd /initrd.img
  ▼
Kernel Linux 6.12 (EFI stub) — descompacta, inicia CPUs, memória, drivers
  ▼
initramfs (initramfs-tools) — carrega virtio_blk, acha a raiz pelo UUID,
  │  verifica (fsck) e monta em /root, depois "switch_root"
  ▼
/sbin/init = systemd (PID 1)
  ├─ systemd-remount-fs, udev, journald
  ├─ osmarcio-firstboot (1º boot: growpart + resize2fs, usuário, senha)
  ├─ systemd-networkd (DHCP) + systemd-resolved (DNS) + timesyncd (NTP)
  ├─ getty@tty1 e serial-getty@ttyAMA0 → "osmarcio-linux login:"
  └─ osmarcio-selftest (só em teste: fw_cfg opt/osmarcio/selftest)
```

## Conceitos

- **UEFI**: firmware moderno; lê partições FAT e executa programas `.EFI`.
  O caminho `EFI/BOOT/BOOTAA64.EFI` é o "padrão de mídia removível": não
  depende de entradas gravadas na NVRAM.
- **ESP** (EFI System Partition): partição FAT32 tipo `EF00`.
- **GRUB**: carrega kernel + initramfs na memória e passa a *linha de
  comando* do kernel.
- **initramfs**: um mini-sistema em RAM, necessário porque o driver do
  disco (virtio_blk) é um módulo que está *no* disco.
- **PID 1**: o primeiro processo de user space; com systemd ele inicia os
  serviços em paralelo, respeitando dependências (`After=`, `Wants=`).

## Comandos para explorar

```bash
systemd-analyze                 # tempo de boot
systemd-analyze blame | head    # serviços mais lentos
systemctl list-units --failed
journalctl -b -u osmarcio-firstboot
cat /proc/cmdline
lsblk -f
```
