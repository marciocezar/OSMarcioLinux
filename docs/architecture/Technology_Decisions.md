# Decisões tecnológicas (índice das ADRs)

| ADR | Tema | Decisão | Status |
|---|---|---|---|
| [0001](../adr/0001-distribuicao-base-debian.md) | Distribuição base | Debian 13 "trixie" | Aceita |
| [0002](../adr/0002-kernel-linux.md) | Kernel | Linux 6.12 LTS do Debian | Aceita |
| [0003](../adr/0003-ambiente-grafico.md) | Desktop | labwc + GTK4 (plano B: XFCE) | Proposta |
| [0004](../adr/0004-imagens-e-instalador.md) | Imagens/instalador | mmdebstrap + GPT/UEFI sem loop; ISO live + Calamares no x86_64 | Aceita (ARM64) / Proposta |
| [0005](../adr/0005-browser-rust-cef.md) | Browser | Rust + CEF via cef-rs | Aceita, POC pendente |
| [0006](../adr/0006-ambiente-de-build.md) | Build | container Debian 13 (Docker) | Aceita |
| [0007](../adr/0007-apps-gtk4-vte.md) | Apps | gtk4-rs 0.11, vte4 0.10, crate core sem GTK | Aceita |

Formato das ADRs: contexto → alternativas → decisão → justificativa →
consequências. Uma decisão nova nunca edita a antiga: cria outra ADR que a
substitui ("Status: Substituída pela ADR N").
