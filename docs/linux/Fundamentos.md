# Fundamentos — o que você precisa saber para entender o OSMarcio Linux

## Linux, kernel e distribuição

- **Kernel** é o programa que controla o hardware e arbitra o acesso a ele:
  agenda processos na CPU, gerencia memória, fala com discos e placas de
  rede, implementa filesystems e o TCP/IP. "Linux", estritamente, é só o
  kernel.
- **Distribuição** é kernel + *user space* (bibliotecas, shell, utilitários,
  serviços) + um gerenciador de pacotes + decisões de configuração. Debian,
  Ubuntu e o OSMarcio Linux são distribuições.
- O antigo projeto **OSMarcio** escrevia o próprio kernel em Rust. O
  **OSMarcio Linux** usa o kernel Linux — e por isso herda 30 anos de
  drivers e compatibilidade.

## Kernel space × user space

A CPU tem níveis de privilégio. No ARM64: **EL1** (kernel) e **EL0**
(programas). Um programa em EL0 não acessa hardware diretamente; ele pede
ao kernel por meio de **syscalls** (`read`, `write`, `openat`, `clone`,
`execve`, `mmap`…). Veja-as acontecendo:

```bash
sudo apt install strace && strace -f -e trace=openat,execve ls /tmp
```

## Processos, threads e memória

- **Processo**: programa em execução com espaço de endereçamento próprio
  (`ps -ef`, `/proc/<pid>/`).
- **Thread**: fluxo de execução que compartilha a memória do processo
  (`ps -eLf`). No Linux ambos são "tasks" criadas por `clone()`.
- **Memória virtual**: cada processo enxerga endereços próprios; a MMU
  traduz para RAM física em páginas de 4 KiB (`cat /proc/self/maps`).

## ABI

A **ABI** (Application Binary Interface) é o contrato binário: números de
syscalls, convenção de chamada, formato ELF, versão da glibc. É por manter
a ABI do Debian 13 que um `.deb` do Debian (ou o `.deb` do VS Code) roda no
OSMarcio Linux sem recompilar.

## Drivers e dispositivos

Drivers ficam no kernel, muitos como **módulos** (`lsmod`, `modinfo
virtio_net`). Em VMs usamos **VirtIO**: dispositivos "paravirtualizados",
simples e rápidos, que o QEMU/UTM expõem.

## Filesystem

Tudo começa em `/` (ext4 no OSMarcio). `/etc` = configuração, `/usr` =
programas, `/var` = dados variáveis, `/home` = usuários, `/boot` = kernel e
initramfs, `/boot/efi` = ESP. `/proc` e `/sys` são filesystems virtuais
que mostram o estado do kernel.

## TCP/IP

A pilha de rede é do kernel. O `systemd-networkd` pede um IP por **DHCP**;
o `systemd-resolved` resolve nomes (**DNS**); o `curl` abre um socket TCP
e negocia **TLS** usando os certificados de `ca-certificates`.
Guia prático: [`docs/networking/Diagnostico_Rede.md`](../networking/Diagnostico_Rede.md).

## Bootloader, init e systemd

Veja [`docs/boot/Boot_Flow.md`](../boot/Boot_Flow.md). Em resumo: firmware
→ bootloader (GRUB) → kernel → initramfs → **init** (PID 1 = systemd), que
sobe os serviços descritos em *unit files* (`systemctl cat
osmarcio-firstboot`).

## Pacotes Debian

Um `.deb` é um arquivo `ar` com `control.tar` (metadados, dependências,
scripts) e `data.tar` (arquivos). `dpkg` instala um `.deb`; **APT** resolve
dependências, baixa de repositórios e verifica as **assinaturas** (arquivos
`InRelease` assinados com as chaves de `debian-archive-keyring`).

```bash
dpkg -L osmarcio-terminal         # arquivos instalados por um pacote
apt-cache policy linux-image-arm64
dpkg-deb --info out/debs/osmarcio-terminal_0.1.0_arm64.deb
```

## Referência educacional: Linux From Scratch

O [LFS](https://www.linuxfromscratch.org/) mostra como compilar cada peça
manualmente. Não o usamos para construir o OSMarcio Linux (seria
impossível manter atualizado), mas é a melhor leitura para entender o que
o `mmdebstrap` automatiza.
