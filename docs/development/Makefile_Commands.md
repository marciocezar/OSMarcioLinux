# Comandos do Makefile (0.3.0)

`make help` lista todos os alvos com descrição. Alvos marcados **[planejado]**
falham com código 2 e explicam a fase que falta — nunca simulam sucesso.
Compila no contêiner Debian 13 (`scripts/build/in-builder.sh`), então funciona
igual no Mac (arm64) e no WSL2/Linux (amd64 nativo).

| Grupo | Alvos | O que fazem |
|---|---|---|
| Diagnóstico | `doctor`, `doctor-mac`, `doctor-wsl`, `doctor-windows`, `version` | verifica ferramentas; mostra a versão da distribuição |
| Terminal | `terminal-build`, `terminal-test`, `terminal-run`, `terminal-deb` | compila; unitários + GUI (Xvfb); executa; empacota |
| Browser | `browser-build`, `browser-test`, `browser-run`, `browser-package`, `browser-version`, `browser-check-updates`, `browser-icons`, `browser-install`, `browser-release` | ver docs/browser |
| Files | `files-build`, `files-test`, `files-run`, `files-package` | compila; 15 testes com operações reais em diretórios temporários; executa; empacota |
| Campo Minado | `minesweeper-build`, `minesweeper-test`, `minesweeper-run`, `minesweeper-package` | compila; 18 testes de regras; executa; empacota |
| Desktop | `desktop-build`, `desktop-test`, `apps-build`, `apps-test`, `apps-lint`, `debs` | todos os .deb; desktop completo na VM; workspace inteiro |
| Imagens | `image-utm`, `verify-images`, `iso-x86_64` [planejado], `image-rockpi4a` [planejado] | IMG ARM64 (chama `desktop-build`); SHA-256 |
| Execução | `run-utm`, `run-qemu-arm64`, `run-qemu-x86_64` [planejado], `run-virtualbox` [planejado] | UTM/QEMU ARM64 |
| Testes | `test-boot`, `test-desktop`, `test-desktop-apps`, `test-browser-hotfix`, `test` | VM real (QEMU/HVF): boot, login, apps |
| Release | `release` [planejado], `release-verify` | release estável bloqueada até x86_64/ROCK Pi validados |

`*-run` só funcionam num Linux com sessão gráfica; no Mac terminam com erro 2
e indicam `make run-utm`/`run-qemu-arm64`. `*-package` e `browser-release`
nunca sobrescrevem um `.deb` existente (incremente `VERSION`).

## Fluxo típico
```bash
make doctor-mac
make apps-lint apps-test
make image-utm
make test-boot
make test-desktop-apps
make run-utm
```
