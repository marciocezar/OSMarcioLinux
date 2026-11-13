#!/usr/bin/env bash
# Escreve <artefato>.manifest.md e <artefato>.sha256.
# uso: write-manifest.sh <artefato relativo> <arch> <plataforma> <perfil> <build-info>
source "$(dirname "$0")/../lib/common.sh"
ART=${1:?}; ARCH=${2:?}; PLATFORM=${3:?}; PROFILE=${4:?}; INFO=${5:?}
cd "$OSM_ROOT"

get() { grep "^$1=" "$INFO" | cut -d= -f2-; }
commit=$(git rev-parse --short HEAD 2>/dev/null || echo "sem-commit")
dirty=$(git status --porcelain 2>/dev/null | grep -q . && echo " (árvore com alterações não commitadas)" || true)

( cd "$(dirname "$ART")" && shasum -a 256 "$(basename "$ART")" > "$(basename "$ART").sha256" )
sha=$(cut -d' ' -f1 < "$ART.sha256")

cat > "$ART.manifest.md" <<EOF
# Manifesto — $(basename "$ART")

| Campo | Valor |
|---|---|
| Distribuição | OSMarcio Linux $OSM_VERSION |
| Base | Debian 13 ($OSM_SUITE) |
| Arquitetura | $ARCH |
| Plataforma | $PLATFORM |
| Perfil | $PROFILE |
| Kernel | $(get kernel) |
| Bootloader | $(get bootloader) |
| systemd | $(get systemd) |
| Pacotes | $(get packages) (lista: \`$(basename "$ART").packages.tsv\`) |
| Data de build (UTC) | $(date -u +%Y-%m-%dT%H:%M:%SZ) |
| Commit | $commit$dirty |
| SHA-256 | \`$sha\` |
| Tamanho | $(du -h "$ART" | cut -f1) em disco (sparse), $(( $(stat -f %z "$ART" 2>/dev/null || stat -c %s "$ART") / 1048576 )) MiB lógicos |

## Resultado dos testes
Ainda não testado. Rode \`make test-utm\` (ou o alvo de teste da plataforma);
o relatório vai para \`reports/tests/\`.

## Limitações
Veja \`PROGRESS.md\` e \`reports/OSMarcioLinux_Platform_Compatibility.md\`.

## Instruções
Veja \`docs/utm/UTM_Guide.md\` (ARM64) ou \`docs/virtualbox/VirtualBox_Guide.md\` (x86_64).
EOF
log "manifesto: $ART.manifest.md"
