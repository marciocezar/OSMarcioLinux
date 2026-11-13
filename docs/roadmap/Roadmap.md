# Roadmap

Ordem obrigatória: infraestrutura antes de dependentes (CEF validado antes
do browser; APT validado antes do Software Center; boot validado antes de
declarar ISO pronta). Estado detalhado e evidências: [PROGRESS.md](../../PROGRESS.md).

| Marco | Conteúdo | Depende de |
|---|---|---|
| **0.1.0** (atual) | Minimal ARM64 bootável e testado; terminal Rust; ADRs; Makefile; setup | — |
| 0.2.0 | Imagem Minimal x86_64 (VDI) no VirtualBox; setup WSL2/Windows validados | Fase 5 |
| 0.3.0 | Perfil Desktop (labwc) no UTM, terminal instalado na imagem | ADR 0003, Fase 6 |
| 0.4.0 | POC CEF (arm64 + amd64) com sandbox → início do Browser | ADR 0005 |
| 0.5.0 | Editor e Files | Fase 6 |
| 0.6.0 | ISO live x86_64 + instalador (Calamares) testado em VDI | Fases 5, 15 |
| 0.7.0 | Software Center (PackageKit), Settings | Fases 11, 13 |
| 0.8.0 | Perfil Developer: Node.js, VS Code, LibreOffice testados | Fase 12 |
| 0.9.0 | ROCK Pi 4A: console + rede (1º marco da placa) | Fase 14, hardware |
| 1.0.0 | Release com manifestos, licenças, testes de regressão | Fase 18 |

Próximos passos concretos (ordem sugerida):

1. Executar `make image-utm` + importar no **app** UTM e registrar evidência.
2. `make rootfs-x86_64` no WSL2 → imagem x86_64 → VDI → VirtualBox.
3. Protótipo labwc no UTM (virtio-gpu) e decisão final da ADR 0003.
4. POC `cef` 156 dentro do OSMarcio Linux ARM64.
