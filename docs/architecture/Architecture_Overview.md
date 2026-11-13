# Visão geral da arquitetura — OSMarcio Linux

> Este documento separa **o que existe (AS-IS)** do **que é planejado (TO-BE)**.
> Diagramas editáveis: [`diagrams/drawio/OSMarcioLinux_Arquitetura.drawio`](../../diagrams/drawio/OSMarcioLinux_Arquitetura.drawio)
> (PNG em `diagrams/png/`).

## 1. O que é o OSMarcio Linux

Uma **distribuição Linux**: o kernel Linux e o *user space* do Debian 13,
mais configuração, identidade e aplicativos próprios em Rust. Não é um
kernel novo nem uma simulação de sistema operacional — por isso roda
qualquer programa Linux compatível com Debian.

## 2. Camadas — AS-IS (v0.1.0, perfil Minimal, ARM64)

```
┌──────────────────────────────────────────────────────────────┐
│ Apps OSMarcio (Rust)   osmarcio-terminal (.deb, ainda fora     │  ← construído e
│                        da imagem: não há desktop no Minimal)  │    testado à parte
├──────────────────────────────────────────────────────────────┤
│ Serviços OSMarcio      osmarcio-firstboot / -cleanup /         │
│                        osmarcio-selftest (systemd units)       │
├──────────────────────────────────────────────────────────────┤
│ Serviços Linux         systemd 257 · networkd (DHCP) ·         │
│                        resolved (DNS) · timesyncd (NTP) · dbus │
├──────────────────────────────────────────────────────────────┤
│ User space             glibc · bash · coreutils · APT 3.0 ·    │
│                        dpkg · sudo · curl · nftables · OpenSSL │
├──────────────────────────────────────────────────────────────┤
│ Kernel                 Linux 6.12.116 LTS (Debian, arm64)      │
├──────────────────────────────────────────────────────────────┤
│ Boot                   UEFI edk2 → GRUB 2.12 (EFI/BOOT/        │
│                        BOOTAA64.EFI) → kernel + initramfs      │
├──────────────────────────────────────────────────────────────┤
│ "Hardware"             QEMU virt (HVF) · VirtIO blk/net/rng    │
│                        — o mesmo modelo de máquina do UTM      │
└──────────────────────────────────────────────────────────────┘
```

## 3. Camadas — TO-BE (v1.0)

| Camada | Componentes planejados | Fase |
|---|---|---|
| Apps OSMarcio | Browser (Rust+CEF), Terminal ✅, Editor, Files, Settings, Software Center, System Monitor, About | 7–13 |
| Apps de terceiros | Node.js, VS Code, LibreOffice, Git, Python… via APT | 12 |
| Desktop | labwc (Wayland) + XWayland, painel, login, notificações (ADR 0003) | 6 |
| Serviços | NetworkManager (desktop), PackageKit, Polkit, PipeWire, UPower | 6, 11, 13 |
| Plataformas | UTM ARM64 ✅ (Minimal) · VirtualBox x86_64 · ROCK Pi 4A | 4, 5, 14 |
| Instalação | IMG pronta (ARM64) ✅ · ISO live + Calamares (x86_64) | 5, 15 |

## 4. Fluxo de build (AS-IS)

```
macOS (host)                       Docker Desktop VM (Linux arm64)
─────────────                      ───────────────────────────────
make image-utm ───────────────►  container osmarcio-linux-builder:trixie
                                   1. mmdebstrap  → rootfs.tar (perfil + overlays)
                                   2. customize-rootfs.sh (hostname, locale,
                                      root bloqueado, serviços, initramfs)
                                   3. mkimage-uefi.sh → disk.img (GPT, ESP, ext4)
images/utm/…img ◄──────────────── + manifesto + SHA-256 + lista de pacotes
make test-utm  → QEMU/HVF no host, 2 boots, autoteste interno → reports/tests/
```

## 5. Componentes do repositório

| Caminho | Responsabilidade |
|---|---|
| `distro/profiles/<perfil>/packages.list` | pacotes de cada perfil (herança minimal → desktop → developer) |
| `distro/overlays/common/` | arquivos copiados para o rootfs (os-release, unidades systemd, scripts) |
| `scripts/build/` | container de build, rootfs, `.deb` |
| `scripts/image/` | montagem da imagem de disco e publicação do artefato |
| `scripts/run/`, `scripts/test/` | execução e testes no QEMU; teste de GUI do terminal |
| `scripts/setup/` | diagnóstico/preparação de macOS, WSL2, Windows |
| `crates/osmarcio-core/` | lógica compartilhada sem GTK |
| `apps/osmarcio-terminal/` | terminal GTK4 + VTE |

## 6. Segurança (AS-IS)

- Root bloqueado; usuário `osmarcio` (grupo `sudo`) criado no 1º boot com
  senha aleatória de troca obrigatória. Nenhuma credencial na imagem.
- `machine-id` e chaves SSH não vão na imagem (gerados por máquina).
- APT com repositórios assinados (`debian-archive-keyring`) — verificado
  no autoteste. `nftables` instalado, **sem regras ativas ainda**.
- Detalhes e pendências: `reports/OSMarcioLinux_Security_Review.md`.
