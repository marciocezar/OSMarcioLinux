# ADR 0005 — OSMarcio Browser: Rust + CEF (cef-rs)

- **Status:** Aceita como direção; **POC pendente** (Fase 10)

## Decisão

O OSMarcio Browser será um executável Rust (`osmarcio-browser`) que embarca
o **Chromium Embedded Framework** via o crate `cef` (projeto
[tauri-apps/cef-rs](https://github.com/tauri-apps/cef-rs)). Interface,
abas, histórico, favoritos, downloads e configurações são código Rust; o
processamento de HTML/CSS/JS continua sendo o Blink/V8 do Chromium (C++).
**O Chromium não é reescrito em Rust**, e não se trata de renomear o
Chromium do Debian.

## Validação inicial (checklist do prompt, item 8.1)

| # | Item | Situação em 2026-11-13 |
|---|---|---|
| 1 | Versão do CEF | crate `cef` **156.4.0+156.0.26** (CEF 156 / Chromium 156), publicado 2026-11-08 — consultado no crates.io |
| 2 | Versão do cef-rs | idem (o crate `cef` é o cef-rs) |
| 3 | Linux AMD64 | **declarado** suportado pelo README do cef-rs — não validado |
| 4 | Linux ARM64 | **declarado** suportado pelo README do cef-rs — não validado |
| 5 | Binários | o build script `cef-dll-sys` baixa o "CEF minimal distribution" do Spotify CDN; alternativa `export-cef-dir` para cache — a verificar checksum |
| 6 | Dependências | a levantar com `ldd libcef.so` na POC (nss, atk, cups, gbm, xkbcommon…) |
| 7 | GTK | CEF no Linux usa janelas próprias (Views) ou *windowless rendering*; integração com GTK4 é o maior risco técnico |
| 8 | Wayland/X11 | Chromium 156 suporta Ozone/Wayland; testar ambos |
| 9 | Multiprocesso | processos browser/renderer/GPU/utility via subprocesso helper — testar |
| 10 | Sandbox | sandbox de namespaces do Chromium; exige `kernel.unprivileged_userns_clone` (padrão Debian ativo) ou `chrome-sandbox` SUID |
| 11 | Redistribuição | CEF: BSD-3; Chromium: BSD + terceiros; codecs proprietários **não** inclusos no build padrão |
| 12 | Atualização de segurança | acompanhar releases do crate `cef` (≈ a cada versão estável do Chromium, 4 semanas) |

Nada além dos itens 1, 2 e 11 foi verificado na prática ainda.

## Critérios para declarar a POC concluída

Executado dentro do OSMarcio Linux (não só compilar):
abrir janela → inicializar CEF → carregar uma página **HTTPS** → renderizar
HTML → executar JavaScript (verificado por retorno ao processo Rust) →
encerrar sem processos órfãos — **com sandbox ativo** (`--no-sandbox`
proibido fora de diagnóstico descartável). Testar em arm64 (UTM) e amd64.

## Riscos

- Tamanho: o CEF tem ~250–300 MB descompactado; não cabe no perfil Minimal.
- ROCK Pi 4A (4 GB RAM, Mali-T860): provável renderização por software;
  medir RAM/tempo de inicialização antes de prometer suporte.
- Integração GTK4: se o *windowless rendering* em textura GTK4 for
  inviável, a janela do browser será uma janela CEF Views com UI própria
  (ainda em Rust), sem GTK.
