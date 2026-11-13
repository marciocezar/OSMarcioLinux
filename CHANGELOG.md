# Changelog

Formato: [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/);
versões em [SemVer](https://semver.org/lang/pt-BR/). A versão é da
distribuição, não do kernel.

## [0.2.1] — não lançada (hotfix do OSMarcio Browser)

### Corrigido
- **Botão X da janela (e Alt+F4, Ctrl+Q) não fechava o OSMarcio Browser**: `can_close` usava `close_browser(0)` e sempre retornava 0 (impasse com o CEF Views). Agora segue o protocolo `try_close_browser()` + `on_window_destroyed`, sem processos órfãos.
- Ctrl+W com várias abas fechava a janela inteira (`DoClose` padrão da BrowserView); agora fecha só a aba.
- Lints `collapsible_if` pré-existentes do browser.

### Adicionado
- Ícone original do OSMarcio Browser (SVG + PNG 16–256), aplicado à janela, barra de tarefas, Alt+Tab, menu e pacote.
- Janela **Sobre o OSMarcio Browser** (menu ☰, F1, `--about`) com versões reais do Browser, Chromium e CEF lidas do `libcef.so` carregado.
- Verificação de atualizações segura (`--check-updates` e botão no Sobre): dpkg/APT/manifesto homologado, sem baixar ou executar binários.
- Pacote `osmarcio-browser_<versão>_<arch>.deb`; alvos `browser-build/run/test/package/version/check-updates/icons/install/release`.
- `scripts/test/qga.py` (qemu-guest-agent) e `make test-browser-hotfix`.

## [0.2.0] — não lançada (em desenvolvimento)

### Adicionado
- Imagem **Desktop ARM64** para UTM: `OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0.img` (LightDM + XFCE, OSMarcio Terminal, OSMarcio Browser, `osmarcio-info`).
- `make run-utm` cria/inicia a VM no app UTM com a IMG como disco VirtIO; `make test-desktop` (login gráfico real, capturas de tela); `make test-boot`, `make browser-bundle`, `make info-deb`.
- Guia `docs/installation/UTM_Quick_Start.md`; relatório `reports/OSMarcioLinux_UTM_Boot_Fix.md`.

### Corrigido
- Boot no UTM: a IMG era anexada como CD/"Boot ISO Image" → UEFI `Shell>` (documentado e automatizado).
- OSMarcio Browser: página carregava mas ficava invisível (BrowserView com largura 0).
- `deb.sh` abortava sem erro para apps sem `.desktop` (pacote do `osmarcio-info` não era gerado).
- Senha temporária do 1º boot agora aparece também na tela de login gráfica.

## [0.1.0] — não lançada (em desenvolvimento)

### Adicionado
- Imagem **Minimal ARM64 UEFI** para UTM/QEMU baseada em Debian 13 e Linux 6.12 LTS.
- Primeiro boot: expansão da raiz, `machine-id`, usuário `osmarcio` com senha aleatória de troca obrigatória.
- Autoteste de boot via `fw_cfg` e teste automatizado de 2 boots (`make test-utm`).
- OSMarcio Terminal (Rust, GTK4 + VTE) com abas e regra da última aba; pacote `.deb`.
- Crate `osmarcio-core` (regras de abas, configuração XDG atômica).
- Container de build Debian 13, Makefile central, scripts de setup (macOS, WSL2, Windows), script de VM VirtualBox.
- ADRs 0001–0007, documentação pedagógica, relatórios, diagrama draw.io (7 abas) com PNG.

### Corrigido (durante o desenvolvimento)
- `firstboot`: SIGPIPE com `pipefail` deixava o usuário sem senha.
- GRUB passou a usar `/vmlinuz`/`/initrd.img` (upgrades de kernel não quebram o boot).
- Terminal: estado da janela era liberado logo após criada (atalhos inativos).
