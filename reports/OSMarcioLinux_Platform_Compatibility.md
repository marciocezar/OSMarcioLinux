# Compatibilidade por plataforma (2026-11-13)

| Plataforma | Arquitetura | Situação | Evidência |
|---|---|---|---|
| QEMU 11 `virt` + HVF (macOS) | arm64 | ✅ **Testado**: boot UEFI, rede, APT, persistência | `reports/tests/boot-arm64-20261113T023007Z.md` |
| App UTM (QEMU backend) | arm64 | ⚠️ Esperado funcionar (mesmo motor/máquina), **não testado no app** | guia em `docs/utm/UTM_Guide.md` |
| UTM com Apple Virtualization | arm64 | ❓ não avaliado | — |
| VirtualBox (Windows) | x86_64 | ⛔ sem imagem ainda | script VM escrito, não testado |
| QEMU x86_64 | x86_64 | ⛔ sem imagem ainda | `mkimage-uefi.sh` aceita amd64 |
| WSL2 (ambiente de build) | x86_64 | ⚠️ scripts escritos, não executados | `scripts/setup/setup-wsl.sh` |
| ROCK Pi 4A (RK3399) | arm64 | ⛔ não iniciado; nenhuma imagem; **a imagem UTM não serve** | ADR 0004 |

Componentes:

| Componente | arm64 | amd64 |
|---|---|---|
| osmarcio-terminal | ✅ build + GUI (Xvfb) + `.deb` | ⏳ não compilado |
| Browser (CEF) | ⛔ POC pendente | ⛔ POC pendente |
| Node.js / VS Code / LibreOffice | ⏳ disponíveis no Debian/Microsoft; não testados | idem |
