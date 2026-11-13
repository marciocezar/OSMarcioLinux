# OSMarcio Linux — ambiente de build isolado (Debian 13 "trixie").
#
# Por que um container?  mmdebstrap/live-build precisam de um kernel Linux
# (chroot, mounts, namespaces). No macOS isso não existe nativamente; no
# Windows usamos WSL2. O container roda dentro da VM do Docker Desktop e
# NUNCA toca discos físicos do host: as imagens são montadas como arquivos.
FROM debian:trixie

ENV DEBIAN_FRONTEND=noninteractive LANG=C.UTF-8

RUN apt-get update && apt-get install -y --no-install-recommends \
        # criação do rootfs
        mmdebstrap debian-archive-keyring ca-certificates arch-test \
        # montagem de imagens sem loop devices
        e2fsprogs dosfstools mtools gdisk fdisk uuid-runtime qemu-utils \
        xz-utils zstd file \
        # ISO (fase x86_64)
        xorriso \
        # Rust + GTK4 + VTE para os aplicativos OSMarcio
        rustup build-essential pkg-config git \
        libgtk-4-dev libvte-2.91-gtk4-dev dpkg-dev \
    && rm -rf /var/lib/apt/lists/*

# Toolchain Rust fixada (mesma de rust-toolchain.toml), instalada pelo rustup
# empacotado no Debian — nada de "curl | sh".
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN rustup toolchain install 1.99.1 --profile minimal -c clippy,rustfmt \
 && rustup default 1.99.1 && rustc --version

# Testes de interface: servidor X virtual + automação de teclado/janelas.
RUN apt-get update && apt-get install -y --no-install-recommends \
        xvfb xdotool x11-utils dbus-x11 libgl1-mesa-dri fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/*

# OSMarcio Browser (CEF): o crate cef-dll-sys compila o libcef_dll_wrapper com
# CMake + Ninja; as bibliotecas abaixo são as dependências de runtime do
# libcef.so (para testar o browser dentro do container, sob Xvfb).
RUN apt-get update && apt-get install -y --no-install-recommends \
        cmake ninja-build curl bc \
        libnss3 libnspr4 libatk1.0-0t64 libatk-bridge2.0-0t64 libcups2t64 \
        libxkbcommon0 libgbm1 libasound2t64 libxcomposite1 libxdamage1 \
        libxrandr2 libxfixes3 libdrm2 libdbus-1-3 libxshmfence1 \
    && rm -rf /var/lib/apt/lists/* \
    # o Chromium recusa rodar como root com sandbox: testes usam este usuário
    && useradd -m -s /bin/bash tester

WORKDIR /work
