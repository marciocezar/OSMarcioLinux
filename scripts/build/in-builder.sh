#!/usr/bin/env bash
# Executa um comando dentro do container de build (cria a imagem se faltar).
source "$(dirname "$0")/../lib/common.sh"
docker image inspect "$OSM_BUILDER_IMAGE" >/dev/null 2>&1 || "$OSM_ROOT/scripts/build/builder-image.sh"
in_builder "$@"
