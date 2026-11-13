#!/usr/bin/env bash
# Valida o XML dos diagramas draw.io e exporta PNG (uma imagem por aba).
source "$(dirname "$0")/../lib/common.sh"
cd "$OSM_ROOT"
python3 scripts/test/docs-check.py --diagrams-only
DRAWIO=$(command -v drawio || ls /Applications/draw.io.app/Contents/MacOS/draw.io 2>/dev/null || true)
if [[ -z $DRAWIO ]]; then
  die "draw.io desktop não encontrado: a exportação PNG precisa dele.
      macOS: brew install --cask drawio   (instalação manual, por sua conta)"
fi
mkdir -p diagrams/png
for f in diagrams/drawio/*.drawio; do
  base=$(basename "$f" .drawio)
  pages=$(grep -o '<diagram ' "$f" | wc -l | tr -d ' ')
  for ((i=1; i<=pages; i++)); do
    "$DRAWIO" --export --format png --scale 2 --border 20 --page-index "$i" \
      --output "diagrams/png/${base}-p$i.png" "$f" >/dev/null 2>&1 \
      || die "falha ao exportar $f página $i"
  done
  log "$f: $pages página(s) exportadas"
done
