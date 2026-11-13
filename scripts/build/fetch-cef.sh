#!/usr/bin/env bash
# Baixa e prepara a distribuição "minimal" do CEF para o OSMarcio Browser.
#
# Por que existe: o crate download-cef 3.0.0 (usado pelo build.rs do
# cef-dll-sys) falha ao ler o índice do CDN do CEF — o index.json passou de
# 10 MB, o limite do cliente HTTP dele. Este script faz o mesmo trabalho:
#   1. lê a versão do CEF do Cargo.lock (crate `cef`, metadado "+156.0.26");
#   2. baixa o índice oficial (cef-builds.spotifycdn.com/index.json);
#   3. baixa o arquivo "minimal" da plataforma e CONFERE o SHA-1 do índice;
#   4. monta o layout esperado pelo cef-dll-sys em out/cef/<os>-<arch>/.
#
# uso: fetch-cef.sh <arm64|amd64>
source "$(dirname "$0")/../lib/common.sh"
need curl; need python3; need shasum; need tar

ARCH=${1:?arch}
case $ARCH in
  arm64) PLATFORM=linuxarm64 ;;
  amd64) PLATFORM=linux64 ;;
  *) die "arch inválida: $ARCH" ;;
esac

DEST="$OSM_ROOT/out/cef/linux-$ARCH"
CRATE_VER=$(awk '/^name = "cef"$/{f=1} f&&/^version/{print; exit}' "$OSM_ROOT/Cargo.lock" | cut -d'"' -f2)
[[ -n $CRATE_VER ]] || die "crate cef não encontrado no Cargo.lock"
CEF_VER=${CRATE_VER#*+}          # 156.4.0+156.0.26 -> 156.0.26
log "crate cef $CRATE_VER -> CEF $CEF_VER ($PLATFORM)"

if [[ -f $DEST/archive.json ]] && grep -q "cef_binary_$CEF_VER+" "$DEST/archive.json"; then
  log "CEF $CEF_VER já preparado em ${DEST#$OSM_ROOT/}"; exit 0
fi

DL="$OSM_ROOT/out/cef/downloads"; mkdir -p "$DL"
INDEX="$DL/index.json"
log "baixando índice do CDN do CEF"
curl -fsSL --retry 3 -o "$INDEX" https://cef-builds.spotifycdn.com/index.json

read -r NAME SHA1 < <(python3 -I - "$INDEX" "$PLATFORM" "$CEF_VER" <<'PY'
import json, sys
idx, plat, ver = sys.argv[1:]
for v in json.load(open(idx))[plat]["versions"]:
    if v["cef_version"].startswith(ver + "+"):
        f = [f for f in v["files"] if f["type"] == "minimal"][0]
        print(f["name"], f["sha1"]); break
else:
    sys.exit(f"versão {ver} não encontrada para {plat}")
PY
)
[[ -n $NAME ]] || die "arquivo não encontrado no índice"
log "arquivo: $NAME"

ARCHIVE="$DL/$NAME"
if [[ ! -f $ARCHIVE ]] || [[ $(shasum -a 1 "$ARCHIVE" | cut -d' ' -f1) != "$SHA1" ]]; then
  curl -fL --retry 3 -o "$ARCHIVE.partial" "https://cef-builds.spotifycdn.com/$NAME"
  mv "$ARCHIVE.partial" "$ARCHIVE"
fi
got=$(shasum -a 1 "$ARCHIVE" | cut -d' ' -f1)
[[ $got == "$SHA1" ]] || die "SHA-1 não confere: esperado $SHA1, obtido $got"
log "SHA-1 conferido: $SHA1"

# Extração e layout (equivalente a download_cef::extract_target_archive).
TMP="$OSM_ROOT/out/cef/extract-$ARCH"
[[ -e $TMP ]] && die "diretório temporário já existe: ${TMP#$OSM_ROOT/} (remova-o manualmente)"
mkdir -p "$TMP"
tar -xjf "$ARCHIVE" -C "$TMP"
SRC="$TMP/${NAME%.tar.bz2}"
[[ -e $DEST ]] && die "${DEST#$OSM_ROOT/} existe com outra versão; mova-o manualmente antes de atualizar"
mv "$SRC/Release" "$DEST"
mv "$SRC/Resources/"* "$DEST/"
for x in CMakeLists.txt cmake include libcef_dll CREDITS.html LICENSE.txt README.txt; do
  [[ -e $SRC/$x ]] && mv "$SRC/$x" "$DEST/"
done
printf '{\n  "type": "minimal",\n  "name": "%s",\n  "sha1": "%s"\n}\n' "$NAME" "$SHA1" > "$DEST/archive.json"
mv "$TMP" "$OSM_ROOT/out/cef/leftover-$ARCH-$(date +%s)"
log "CEF pronto em ${DEST#$OSM_ROOT/} ($(du -sh "$DEST" | cut -f1))"
