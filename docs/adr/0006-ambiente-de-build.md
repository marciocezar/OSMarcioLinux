# ADR 0006 — Ambiente de build: container Debian 13 (Docker)

- **Status:** Aceita (2026-11-13)

## Contexto

`mmdebstrap`, `chroot`, `update-initramfs`, `grub-mkimage` e os pacotes
`-dev` do GTK4/VTE são ferramentas Linux. O host principal é um Mac Apple
Silicon; o secundário, Windows + WSL2. O prompt proíbe fingir que
live-build roda nativamente no macOS e exige não misturar builds.

## Decisão

Todo build Linux roda em **um único container** `osmarcio-linux-builder:trixie`
(`build/docker/Dockerfile.builder`):

- mesmas versões de bibliotecas que a distro final (Debian 13) → os `.deb`
  linkam contra exatamente o que estará instalado;
- toolchain Rust fixada (`rust-toolchain.toml` = 1.99.1) instalada pelo
  `rustup` empacotado no Debian (sem `curl | sh`);
- target Rust do container em `out/target-linux-<arch>/`, separado do
  `target/` do host;
- cache do registry Cargo em um volume Docker (`osmarcio-cargo-registry`).

O container usa `--privileged` (necessário para `chroot` + montagens de
`/proc` do mmdebstrap). Isso dá privilégios **dentro da VM do Docker
Desktop**, não sobre o macOS/Windows; e os scripts de imagem não usam loop
devices, portanto não tocam em discos.

## Consequências

- No Mac o container é arm64 nativo → imagens ARM64 em ~1 min.
- Imagens amd64 no Mac exigem emulação (binfmt/QEMU do Docker Desktop):
  funcionam em tese, mas lentas — o build x86_64 "oficial" será no WSL2.
- QEMU de **teste** roda no host (macOS) para usar a aceleração HVF.
