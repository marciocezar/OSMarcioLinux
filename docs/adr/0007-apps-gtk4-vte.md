# ADR 0007 — Apps OSMarcio: Rust + gtk4-rs; terminal com VTE

- **Status:** Aceita (2026-11-13)

## Decisão

- Interface: **gtk4-rs 0.11** (feature `v4_18`, porque o Debian 13 traz
  GTK 4.18.6). libadwaita fica opcional, por app.
- Terminal: **vte4 0.10** (VTE 0.80 no Debian 13) — a mesma biblioteca do
  GNOME Terminal. Não reimplementamos emulação VT/ANSI nem PTY.
- Lógica sem GTK no crate **`osmarcio-core`** (regras de abas, configuração
  XDG com gravação atômica), testável em qualquer host com `cargo test`.
- Empacotamento: `.deb` gerado por `scripts/build/deb.sh` com `dpkg-deb`
  (metadados em `[package.metadata.osmarcio-deb]` de cada app).
- Licença do código próprio: GPL-3.0-or-later (compatível com GTK/VTE,
  LGPL) — **a confirmar pelo dono do projeto**.

## Nota técnica registrada durante a implementação

O crate `vte4` 0.10 referencia `gtk::Accessible`, que no `gtk4` 0.11 só
existe com a feature `v4_10`+. Sem habilitar uma feature de versão o build
falha com `E0425`. Habilitamos `v4_18` (versão real do Debian 13).
