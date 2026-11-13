#!/usr/bin/env bash
# Verifica o SHA-256 de cada artefato em images/ contra seu arquivo .sha256.
source "$(dirname "$0")/../lib/common.sh"
cd "$OSM_ROOT/images"
n=0; bad=0
while IFS= read -r f; do
  n=$((n+1))
  if ! ( cd "$(dirname "$f")" && shasum -a 256 -c "$(basename "$f")" ); then
    bad=$((bad+1))
  fi
done < <(find . -name '*.sha256' | sort)
[[ $n -gt 0 ]] || die "nenhum artefato com .sha256 em images/"
[[ $bad -eq 0 ]] || die "$bad artefato(s) com checksum inválido"
log "$n artefato(s) verificados"
