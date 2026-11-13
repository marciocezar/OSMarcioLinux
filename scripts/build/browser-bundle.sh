#!/usr/bin/env bash
# Monta o diretório de execução do OSMarcio Browser: binário Rust + runtime
# do CEF lado a lado (o CEF procura libcef.so, *.pak, icudtl.dat e
# v8_context_snapshot.bin no diretório do executável).
# Roda DENTRO do container. uso: browser-bundle.sh <destino> [debug|release]
source "$(dirname "$0")/../lib/common.sh"
DEST=${1:?destino}; MODE=${2:-release}
ARCH=$(dpkg --print-architecture)
CEF="${CEF_PATH:-$OSM_ROOT/out/cef/linux-$ARCH}"
[[ -f $CEF/libcef.so ]] || die "CEF não encontrado em $CEF — rode: scripts/build/fetch-cef.sh $ARCH (no host)"
TARGET=${CARGO_TARGET_DIR:-$OSM_ROOT/target}

flag=(); [[ $MODE == release ]] && flag=(--release)
cargo build "${flag[@]}" -p osmarcio-browser --locked

rm -rf "$DEST.partial"; mkdir -p "$DEST.partial/locales"
install -m755 "$TARGET/$MODE/osmarcio-browser" "$DEST.partial/"
for f in libcef.so libvk_swiftshader.so libvulkan.so.1 vk_swiftshader_icd.json \
         icudtl.dat v8_context_snapshot.bin resources.pak chrome_100_percent.pak \
         chrome_200_percent.pak chrome-sandbox LICENSE.txt CREDITS.html archive.json; do
  [[ -e $CEF/$f ]] && cp "$CEF/$f" "$DEST.partial/"
done
cp "$CEF"/locales/pt-BR*.pak "$CEF"/locales/en-US*.pak "$DEST.partial/locales/"
# libcef.so vem com símbolos de depuração (~2,5 GB); remove-os (~10x menor).
strip --strip-unneeded "$DEST.partial/libcef.so" "$DEST.partial/libvk_swiftshader.so"
# chrome-sandbox só é usado se o kernel não permitir user namespaces; nesse
# caso precisa ser SUID root (definido no pacote .deb, nunca no bundle).
chmod 0755 "$DEST.partial/chrome-sandbox"
rm -rf "$DEST"; mv "$DEST.partial" "$DEST"
log "bundle: ${DEST#$OSM_ROOT/} ($(du -sh "$DEST" | cut -f1))"
