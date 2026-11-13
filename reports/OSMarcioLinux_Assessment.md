# Assessment — Fase 1 (2026-11-13)

## Ambiente host verificado

| Item | Valor |
|---|---|
| Host | macOS 27.2.1, Apple Silicon (arm64), 10 CPUs, 24 GiB |
| Docker Desktop | 29.9.3, VM aarch64, 8 GiB |
| QEMU | 11.0.1 (Homebrew), firmware edk2 aarch64/x86_64 incluídos, HVF disponível |
| UTM | instalado (`/Applications/UTM.app`, `utmctl` presente) |
| Rust (host) | 1.99.1; no container: 1.99.1 via rustup do Debian |
| draw.io desktop | instalado (exportação PNG funcional) |
| Ausentes no host | xorriso, mtools, sgdisk (usados só no container — não precisam estar no Mac) |

Windows/WSL2 e a ROCK Pi 4A **não estavam disponíveis** nesta sessão.

## Decisões

| Tema | Decisão | ADR |
|---|---|---|
| Base | Debian 13 trixie (estável vigente) | 0001 |
| Kernel | 6.12.116 LTS do Debian | 0002 |
| Desktop | labwc + GTK4 (proposta; plano B XFCE) | 0003 |
| Imagens | mmdebstrap + GPT/UEFI sem loop; ISO live + Calamares no x86_64 | 0004 |
| Browser | Rust + CEF (`cef` 156.4.0); POC pendente | 0005 |
| Build | container Debian 13 | 0006 |
| Apps | gtk4-rs 0.11 + vte4 0.10 | 0007 |

## Validação do CEF (resumo — detalhes na ADR 0005)

Verificado: versão do crate (`cef` 156.4.0+156.0.26, 2026-11-08), suporte
declarado a Linux x86_64 e ARM64, licença BSD-3. **Não verificado**:
download/checksum dos binários, dependências em runtime, integração GTK4,
Wayland, sandbox, consumo de memória. O Browser **não** foi iniciado,
conforme a regra "não construir o Browser antes de validar o CEF".
