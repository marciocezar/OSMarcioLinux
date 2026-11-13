# ADR 0002 — Kernel: Linux 6.12 LTS empacotado pelo Debian

- **Status:** Aceita (2026-11-13)

## Contexto

O sistema usa um kernel Linux real. Não reimplementamos processos, memória,
filesystems, TCP/IP ou drivers — e não reutilizamos o kernel do projeto
OSMarcio antigo (que permanece intocado em outro diretório).

## Decisão

Usar o **kernel oficial do Debian 13**, metapacotes `linux-image-arm64` e
`linux-image-amd64`. Versão no build atual: **6.12.116+deb13** (série
6.12, que é *longterm* em kernel.org).

Não compilaremos kernel próprio enquanto não houver necessidade concreta.
Candidato conhecido a necessidade: **ROCK Pi 4A (RK3399)** — se o kernel
Debian não tiver o necessário para HDMI/GPU/áudio da placa, uma ADR nova
documentará config/patches em `kernel/` (Fase 14).

## Justificativa

- Segurança: o Debian publica correções do kernel via `trixie-security`;
  o `apt upgrade` do usuário atualiza o kernel junto com o resto.
- Compatibilidade: drivers VirtIO (UTM/QEMU), `vboxvideo`/VMSVGA, e o
  device tree do RK3399 (`rk3399-rock-pi-4a.dtb`) já fazem parte do kernel
  arm64 do Debian.
- "Mais novo" não é critério: a 6.12 é LTS e é a que o Debian mantém.

## Configuração aplicada pelo OSMarcio

- `initramfs-tools` com módulos virtio forçados (`virtio_pci`, `virtio_blk`,
  `virtio_scsi`, `virtio_net`) — ver `scripts/build/customize-rootfs.sh`.
- Linha de comando (GRUB): `root=UUID=… ro console=tty0 console=ttyAMA0,115200`
  no ARM64 (o último `console=` é o console principal; o serial facilita
  testes automatizados, e a tela do UTM continua recebendo as mensagens).

## Atualização

`sudo apt update && sudo apt full-upgrade` instala o novo kernel. O
`grub.cfg` (gerado pelo nosso script, não pelo `update-grub`) aponta para
`/vmlinuz` e `/initrd.img` — symlinks que o pacote `linux-base` do Debian
atualiza a cada kernel instalado. Assim o kernel novo é usado no próximo
boot sem regenerar o `grub.cfg` (verificado no autoteste: `grub-symlink`).

Limitação: a entrada "kernel anterior" (`/vmlinuz.old`) só aparece se o
symlink existir na hora do build da imagem; numa imagem nova ela não
existe. Evolução prevista: GRUB gerenciado pelo Debian (`grub-install
--removable` + `update-grub`) ou `systemd-boot` com `kernel-install`.
