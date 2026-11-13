# Referência do Makefile

`make help` lista tudo. Situação de cada alvo em 2026-11-13:

| Alvo | O que faz | Situação |
|---|---|---|
| `help`, `version` | ajuda / versão da distro | ✅ |
| `doctor`, `doctor-mac` | diagnóstico do host | ✅ testado no Mac |
| `setup-mac` | diagnóstico + oferece `brew install` item a item | ✅ (instalação nunca silenciosa) |
| `doctor-wsl`, `setup-wsl` | idem no WSL2 (apt) | ⚠️ escrito, não executado em WSL2 |
| `doctor-windows`, `setup-windows` | diagnóstico do Windows (PowerShell) | ⚠️ escrito, não executado |
| `builder` | constrói o container Debian 13 | ✅ |
| `apps-build`, `apps-lint`, `apps-test`, `test-apps` | workspace Rust no container | ✅ |
| `terminal-build`, `terminal-test`, `terminal-deb`, `debs` | terminal | ✅ |
| `rootfs-arm64` | só o rootfs `.tar` | ✅ (via `image-utm`) |
| `rootfs-x86_64` | rootfs amd64 | ⚠️ script suporta; não executado |
| `image-utm`, `image-minimal`, `build-arm64`, `distro-build` | IMG ARM64 UEFI + manifesto | ✅ |
| `verify-images`, `checksums`, `release-verify` | confere SHA-256 | ✅ |
| `run-qemu-arm64` | QEMU com janela, disco persistente em `out/vm/` | ✅ |
| `run-utm` | abre o UTM e indica o guia de importação | ⚠️ importação manual |
| `test-utm`, `test-network` | 2 boots reais + autoteste interno | ✅ |
| `test`, `test-regression` | lint + apps + boot | ✅ |
| `docs`, `docs-check` | links internos + XML dos diagramas | ✅ |
| `diagrams` | valida e exporta PNG (draw.io desktop) | ✅ |
| `clean` | apaga `out/target-*`, `out/test-*`, `out/debs`, `target` — **nunca** `out/vm/` nem `images/` | ✅ |
| `browser-*`, `editor-build`, `files-build`, `settings-build`, `software-center-build` | — | ⛔ planejado (erro claro, código 2) |
| `image-desktop`, `image-developer`, `build-x86_64`, `iso-x86_64`, `image-rockpi4a` | — | ⛔ planejado |
| `run-qemu-x86_64`, `run-virtualbox`, `test-browser`, `test-install`, `test-virtualbox`, `release` | — | ⛔ planejado |

Variáveis: `PROFILE=minimal|desktop|developer` (padrão `minimal`),
`UTM_IMG=<caminho>` para escolher a imagem de `run-*`/`test-utm`,
`TIMEOUT=<s>` para o teste de boot.

Regras embutidas: nenhum alvo sobrescreve uma imagem existente
(`refuse_overwrite`), nenhum alvo usa dispositivos de bloco do host, e
nenhum instala software sem perguntar.
