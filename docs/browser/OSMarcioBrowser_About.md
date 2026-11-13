# OSMarcio Browser — Tela Sobre

Abrir: menu **☰ → Sobre o OSMarcio Browser**, tecla **F1**, ou
`osmarcio-browser --about` (também no clique direito do ícone, ação do `.desktop`).
Sem janela: `osmarcio-browser --version`.

Janela própria (não modal: o navegador continua utilizável), com o ícone
original, texto selecionável e os botões **Verificar atualizações**,
**Copiar informações** e **Fechar**. O conteúdo é uma página interna
(`data:` URL) desenhada pelo motor, que segue o tema claro/escuro
(`color-scheme`).

## Campos e origem (nada é fixo no código)

| Campo | Origem real | Exemplo medido (0.2.1, ARM64) |
|---|---|---|
| Versão do Browser | `CARGO_PKG_VERSION` (workspace) | 0.2.1 |
| Chromium Engine | `cef_version_info(4..7)` do **libcef.so carregado** | 156.0.8078.88 |
| CEF Runtime | `cef_version_info(0..2)` do libcef.so carregado | 156.0.26 |
| CEF (compilado) | `cef_dll_sys::CEF_VERSION` (headers do build) | 156.0.26+g8ea0c73+chromium-156.0.8078.88 |
| Bindings Rust | versão do crate `cef` no `Cargo.lock` (build.rs) | 156.4.0+156.0.26 |
| Arquitetura | `std::env::consts::ARCH` | aarch64 |
| Sistema / versão | `/etc/os-release` (`NAME`, `VERSION_ID`) | OSMarcio Linux / 0.2.1 |
| Licenças | GPL-3.0-or-later (app); BSD-3-Clause + terceiros (Chromium/CEF, `CREDITS.html`) | — |
| Estado de atualização | botão → [Atualizações](OSMarcioBrowser_Updates.md) | — |

`cef_version_info` é exportado pelo `libcef.so`, mas o cef-rs 156.4.0 não tem
binding; `src/version.rs` declara o FFI (`int cef_version_info(int)`), único
trecho `unsafe` permitido do app. Se o runtime carregado divergir do compilado,
a linha "CEF (compilado)" mostra **⚠ difere do runtime**. Um dado ausente
aparece como **"Não disponível"**.

## Testes
`test-browser-hotfix.sh sobre-abrir|sobre-verificar|sobre-fechar`: abre com F1;
versão do Chromium no Sobre == `--version`; `runtime_confere=true`; botão
Verificar mostra o estado; Copiar → texto colado na barra de endereços contém
"Chromium Engine"; Fechar fecha só o Sobre; X com o Sobre aberto fecha tudo.
