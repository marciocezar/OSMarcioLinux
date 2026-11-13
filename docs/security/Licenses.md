# Licenças

| Componente | Licença | Observação |
|---|---|---|
| Código próprio (apps Rust, scripts, configs) | GPL-3.0-or-later | escolha inicial, **a confirmar** pelo dono do projeto |
| Kernel Linux | GPL-2.0 | binário do Debian; fonte em `apt source linux` |
| Pacotes Debian (`main`) | DFSG (várias) | lista em `<imagem>.packages.tsv`; textos em `/usr/share/doc/*/copyright` |
| `non-free-firmware` | várias, redistribuíveis | habilitado para firmware de hardware real (ROCK Pi, Wi-Fi) |
| GRUB 2 | GPL-3.0 | |
| GTK 4, VTE | LGPL-2.1+ / LGPL-3.0+ | ligação dinâmica |
| Crates Rust (gtk4-rs, vte4-rs, serde, toml…) | MIT / Apache-2.0 | conferir com `cargo license` antes de release |
| CEF (futuro) | BSD-3-Clause | Chromium: BSD + licenças de terceiros (`about:credits`); sem codecs proprietários no build padrão |
| VS Code (opcional, futuro) | licença Microsoft (binário) | **não** redistribuído na imagem: instalado pelo usuário a partir do repositório da Microsoft |

Pendências antes de qualquer release: gerar inventário de licenças dos
crates e dos pacotes incluídos (Fase 18).
