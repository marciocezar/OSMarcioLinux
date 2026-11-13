#!/usr/bin/env bash
# Constrói (ou reutiliza) a imagem Docker do ambiente de build.
source "$(dirname "$0")/../lib/common.sh"
need docker
docker info >/dev/null 2>&1 || die "Docker não está rodando. Abra o Docker Desktop e tente de novo."
log "construindo $OSM_BUILDER_IMAGE"
docker build -t "$OSM_BUILDER_IMAGE" -f "$OSM_ROOT/build/docker/Dockerfile.builder" "$OSM_ROOT/build/docker"
