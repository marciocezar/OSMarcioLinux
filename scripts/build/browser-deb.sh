#!/usr/bin/env bash
# Gera out/debs/osmarcio-browser_<versão>_<arch>.deb — Browser Rust + runtime
# CEF homologado (o mesmo contra o qual o binário foi compilado), ícones,
# .desktop e conffile do canal de atualizações. Roda DENTRO do container.
#
# Layout instalado:
#   /usr/lib/osmarcio-browser/      binário + libcef.so + recursos do Chromium
#   /usr/bin/osmarcio-browser       -> ../lib/osmarcio-browser/osmarcio-browser
#   /usr/share/applications/osmarcio-browser.desktop
#   /usr/share/icons/hicolor/{16..256,scalable}/apps/osmarcio-browser.{png,svg}
#   /etc/osmarcio/browser-updates.toml   (conffile)
# Nunca sobrescreve um .deb existente com o mesmo nome.
source "$(dirname "$0")/../lib/common.sh"
APP=osmarcio-browser
ARCH=$(dpkg --print-architecture)
SRC="$OSM_ROOT/apps/$APP"
OUTDIR="$OSM_ROOT/out/debs"; mkdir -p "$OUTDIR"
DEB="$OUTDIR/${APP}_${OSM_VERSION}_${ARCH}.deb"
refuse_overwrite "$DEB"

BUNDLE="$OSM_ROOT/out/browser-bundle-$ARCH-v$OSM_VERSION"
"$OSM_ROOT/scripts/build/browser-bundle.sh" "$BUNDLE" release
"$BUNDLE/osmarcio-browser" --version >&2 || die "o binário não executa --version"

STAGE=$(mktemp -d); trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/usr/lib" "$STAGE/usr/bin"
cp -a "$BUNDLE" "$STAGE/usr/lib/$APP"
ln -s "../lib/$APP/$APP" "$STAGE/usr/bin/$APP"
install -Dm644 "$SRC/packaging/$APP.desktop" "$STAGE/usr/share/applications/$APP.desktop"
for d in "$SRC"/assets/icons/hicolor/*/apps; do
  sz=$(basename "$(dirname "$d")")
  for f in "$d"/$APP.*; do install -Dm644 "$f" "$STAGE/usr/share/icons/hicolor/$sz/apps/$(basename "$f")"; done
done
install -Dm644 "$SRC/packaging/browser-updates.toml" "$STAGE/etc/osmarcio/browser-updates.toml"
install -Dm644 "$OSM_ROOT/LICENSE" "$STAGE/usr/share/doc/$APP/copyright"

meta() { sed -n '/^\[package.metadata.osmarcio-deb\]/,/^\[/p' "$SRC/Cargo.toml" | grep "^$1 *=" | cut -d'"' -f2 || true; }
mkdir -p "$STAGE/DEBIAN"
echo "/etc/osmarcio/browser-updates.toml" > "$STAGE/DEBIAN/conffiles"
cat > "$STAGE/DEBIAN/control" <<CTL
Package: $APP
Version: $OSM_VERSION
Architecture: $ARCH
Maintainer: OSMarcio Linux <osmarcio-linux@localhost>
Section: $(meta section)
Priority: optional
Depends: $(meta depends), curl
Installed-Size: $(du -sk "$STAGE" | cut -f1)
Description: OSMarcio Browser — navegador em Rust com motor Chromium (CEF)
 Inclui o runtime do Chromium Embedded Framework homologado para esta versão
 ($(grep -o 'cef_binary_[^"]*' "$BUNDLE/archive.json" | head -1)).
 Parte do OSMarcio Linux $OSM_VERSION.
CTL
log "empacotando (xz) — pode levar alguns minutos"
dpkg-deb --root-owner-group -Zxz -z6 --build "$STAGE" "$DEB.partial" >/dev/null
mv "$DEB.partial" "$DEB"
log "pacote: ${DEB#$OSM_ROOT/} ($(du -h "$DEB" | cut -f1))"
dpkg-deb --info "$DEB" | sed -n '/Package:/,$p' >&2
