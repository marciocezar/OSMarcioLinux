# ADR 0001 — Distribuição base: Debian 13 "trixie"

- **Status:** Aceita (2026-11-13)
- **Contexto:** Fase 1 (Assessment)

## Contexto

O OSMarcio Linux precisa ser uma distribuição Linux **real**: instalar e rodar
programas Linux convencionais (Node.js, VS Code, LibreOffice, Git, Python),
receber atualizações de segurança e rodar em ARM64 (UTM, ROCK Pi 4A) e x86_64
(VirtualBox). O projeto também é educacional, então a base precisa ser
compreensível e bem documentada.

## Alternativas avaliadas

| Critério | Debian 13 | Ubuntu 24.04 LTS | Buildroot | Yocto | LFS |
|---|---|---|---|---|---|
| APT/dpkg + repositório binário enorme | ✅ | ✅ | ❌ (sem gerenciador de pacotes em runtime) | ⚠️ (opkg/rpm, repositório próprio) | ❌ |
| arm64 + amd64 oficiais | ✅ | ✅ | ✅ | ✅ | ⚠️ manual |
| Atualizações de segurança mantidas | ✅ (security.debian.org) | ✅ | ❌ (você mantém) | ❌ (você mantém) | ❌ |
| Chromium/CEF, GTK4, VTE, LibreOffice prontos | ✅ | ✅ | ⚠️ | ⚠️ | ❌ |
| VS Code (.deb oficial amd64/arm64) | ✅ | ✅ | ❌ | ❌ | ❌ |
| Ferramentas de build de imagem | mmdebstrap, live-build, debian-installer | idem + snaps | próprio | bitbake | manual |
| Sem pacotes forçados (snap) | ✅ | ❌ (snap no Firefox/Chromium) | ✅ | ✅ | ✅ |
| Tamanho mínimo razoável | ✅ (~600 MiB com kernel) | ⚠️ maior | ✅✅ | ✅✅ | ✅ |
| Valor educacional | alto | alto | alto (embarcado) | médio (complexo) | altíssimo, mas impraticável para manter |

## Decisão

Usar **Debian 13 "trixie"** (estável) como base, com os repositórios
`main contrib non-free-firmware`, `trixie-updates` e `trixie-security`.
A versão foi verificada no build: o mmdebstrap baixou a suíte `trixie`
atual em 2026-11-13 (kernel 6.12.116, systemd 257.13, APT 3.0.4).

## Justificativa

- É o único candidato que atende ao mesmo tempo: APT, segurança mantida,
  arm64+amd64 oficiais, sem snaps e com `mmdebstrap` — que gera um rootfs
  reprodutível sem precisar de VM.
- O Debian 13 é a estável vigente; o Debian 14 "forky" ainda está em
  desenvolvimento (não usar "testing" em uma distro que promete estabilidade).

## Consequências

- ✅ Qualquer `apt install` de pacote Debian funciona no OSMarcio Linux
  (comprovado: `apt-get install hello` no autoteste de boot).
- ✅ O `ID_LIKE=debian` no `/etc/os-release` faz instaladores de terceiros
  (ex.: VS Code) reconhecerem o sistema.
- ⚠️ O OSMarcio Linux herda o ciclo de vida do Debian 13 (suporte até ~2028,
  LTS até ~2030). A migração para o Debian 14 será uma ADR futura.
- ⚠️ Não usamos repositórios próprios ainda: pacotes OSMarcio (`.deb`) são
  instalados na imagem no build. Um repositório APT assinado próprio é
  trabalho futuro (Fase 18).

## Alternativas descartadas

- **Ubuntu LTS:** bom, mas snaps forçados para navegadores e base maior.
- **Buildroot / Yocto:** ótimos para firmware embarcado; não oferecem
  instalação de programas em runtime — contradiz o objetivo principal.
- **LFS:** usado apenas como **referência educacional** em `docs/linux/`.
