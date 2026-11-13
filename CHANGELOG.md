# Changelog

Formato: [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/);
versões em [SemVer](https://semver.org/lang/pt-BR/). A versão é da
distribuição, não do kernel.

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
