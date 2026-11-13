# OSMarcio Linux

Distribuição Linux própria, baseada em **Debian 13** com **kernel Linux 6.12
LTS**, com aplicativos próprios em **Rust**. Projeto educacional e
incremental, independente do antigo OSMarcio (kernel em Rust), que fica
intocado em outro diretório.

> **Estado (0.3.0, em desenvolvimento):** a imagem **Desktop ARM64**
> (`images/utm/OSMarcioLinux-UTM-AArch64-Desktop-v0.3.0.img`) inicializa por
> UEFI até **LightDM → XFCE** com quatro aplicativos próprios em Rust:
> **OSMarcio Terminal** (GTK4 + VTE), **OSMarcio Browser** (Rust + CEF),
> **OSMarcio Files** (GTK4 + GIO) e **Campo Minado** (GTK4) — testados no
> QEMU/HVF (`make test-desktop-apps`). A `.img` é um **disco**, não uma ISO
> ([UTM Quick Start](docs/installation/UTM_Quick_Start.md)).
> x86_64/VirtualBox e ROCK Pi 4A ainda **não** existem.
> Detalhes honestos em [PROGRESS.md](PROGRESS.md).

## Começo rápido (Mac Apple Silicon)

```bash
make doctor-mac
```

```bash
make image-utm
```

```bash
make test-boot
```

```bash
make test-desktop
```

```bash
make run-utm
```

`make run-utm` cria (só na 1ª vez) e inicia a VM no app UTM — passo a passo
manual em [docs/installation/UTM_Quick_Start.md](docs/installation/UTM_Quick_Start.md).
Sem UTM: `make run-qemu-arm64`. No primeiro boot a tela de login mostra a
senha temporária do usuário `osmarcio`; trocá-la é obrigatório.

Apps Rust (compilam no container Linux):

```bash
make apps-test
```

```bash
make terminal-deb
```

`make help` lista todos os alvos ([referência](docs/build/Makefile_Reference.md)).

## Onde está cada coisa

| | |
|---|---|
| Arquitetura | [docs/architecture/Architecture_Overview.md](docs/architecture/Architecture_Overview.md) |
| Decisões (ADRs) | [docs/architecture/Technology_Decisions.md](docs/architecture/Technology_Decisions.md) |
| Fundamentos de Linux | [docs/linux/Fundamentos.md](docs/linux/Fundamentos.md) |
| Como o sistema inicializa | [docs/boot/Boot_Flow.md](docs/boot/Boot_Flow.md) |
| Rede | [docs/networking/Diagnostico_Rede.md](docs/networking/Diagnostico_Rede.md) |
| Terminal | [docs/terminal/OSMarcio_Terminal.md](docs/terminal/OSMarcio_Terminal.md) |
| Browser | [docs/browser/OSMarcioBrowser_Architecture.md](docs/browser/OSMarcioBrowser_Architecture.md) · [hotfix 0.2.1](reports/OSMarcioBrowser_Fix_Report.md) · [maximizar](docs/browser/OSMarcioBrowser_Window_Management.md) |
| Files | [arquitetura](docs/applications/OSMarcioFiles_Architecture.md) · [guia](docs/applications/OSMarcioFiles_User_Guide.md) |
| Campo Minado | [arquitetura](docs/applications/OSMarcioMinesweeper_Architecture.md) · [guia](docs/applications/OSMarcioMinesweeper_User_Guide.md) |
| Apps do desktop | [arquitetura](docs/architecture/Desktop_Application_Architecture.md) · [testes](docs/testing/Desktop_Application_Test_Plan.md) · [Makefile](docs/development/Makefile_Commands.md) · [problemas](docs/troubleshooting/Desktop_Troubleshooting.md) |
| Testes | [docs/testing/Test_Strategy.md](docs/testing/Test_Strategy.md) |
| Roadmap | [docs/roadmap/Roadmap.md](docs/roadmap/Roadmap.md) |
| Relatórios | [reports/](reports/) |
| Diagramas | [diagrams/drawio/](diagrams/drawio/) (editável) · [diagrams/png/](diagrams/png/) |

## Estrutura

```
apps/            aplicativos Rust (terminal, browser, files, minesweeper, info)
crates/          bibliotecas Rust compartilhadas (osmarcio-core)
distro/          perfis de pacotes e overlays do rootfs
build/docker/    container de build Debian 13
scripts/         setup, build, image, run, test, release, windows
images/          artefatos versionados (+ .sha256, manifesto, pacotes)
docs/ reports/ diagrams/
out/             temporários de build, discos de VM (out/vm — nunca apagado pelo make)
```

Licença do código próprio: GPL-3.0-or-later (provisória) — ver [LICENSE](LICENSE).
