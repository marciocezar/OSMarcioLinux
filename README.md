# OSMarcio Linux

Distribuição Linux própria, baseada em **Debian 13** com **kernel Linux 6.12
LTS**, com aplicativos próprios em **Rust**. Projeto educacional e
incremental, independente do antigo OSMarcio (kernel em Rust), que fica
intocado em outro diretório.

> **Estado (0.1.0, em desenvolvimento):** a imagem **Minimal ARM64** inicializa
> por UEFI, tem rede, APT e persistência — testada no QEMU com HVF (o motor
> do UTM). O **OSMarcio Terminal** funciona e é empacotado em `.deb`.
> Desktop, browser, x86_64/VirtualBox e ROCK Pi 4A ainda **não** existem.
> Detalhes honestos em [PROGRESS.md](PROGRESS.md).

## Começo rápido (Mac Apple Silicon)

```bash
make doctor-mac
```

```bash
make image-utm
```

```bash
make test-utm
```

```bash
make run-qemu-arm64
```

Depois, para o app UTM: [docs/utm/UTM_Guide.md](docs/utm/UTM_Guide.md).
No primeiro boot a tela de login mostra a senha temporária do usuário
`osmarcio`; trocá-la é obrigatório.

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
| Testes | [docs/testing/Test_Strategy.md](docs/testing/Test_Strategy.md) |
| Roadmap | [docs/roadmap/Roadmap.md](docs/roadmap/Roadmap.md) |
| Relatórios | [reports/](reports/) |
| Diagramas | [diagrams/drawio/](diagrams/drawio/) (editável) · [diagrams/png/](diagrams/png/) |

## Estrutura

```
apps/            aplicativos Rust (osmarcio-terminal)
crates/          bibliotecas Rust compartilhadas (osmarcio-core)
distro/          perfis de pacotes e overlays do rootfs
build/docker/    container de build Debian 13
scripts/         setup, build, image, run, test, release, windows
images/          artefatos versionados (+ .sha256, manifesto, pacotes)
docs/ reports/ diagrams/
out/             temporários de build, discos de VM (out/vm — nunca apagado pelo make)
```

Licença do código próprio: GPL-3.0-or-later (provisória) — ver [LICENSE](LICENSE).
