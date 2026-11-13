#!/usr/bin/env bash
# Gera o pacote .deb de um aplicativo OSMarcio. Roda DENTRO do container
# (a arquitetura do pacote é a do container: arm64 no Mac, amd64 no WSL2).
#
# uso: deb.sh <app>      ex.: deb.sh osmarcio-terminal
source "$(dirname "$0")/../lib/common.sh"
APP=${1:?app}
MAN="$OSM_ROOT/apps/$APP/Cargo.toml"
[[ -f $MAN ]] || die "app inexistente: $APP"

meta() { sed -n '/^\[package.metadata.osmarcio-deb\]/,/^\[/p' "$MAN" | grep "^$1 *=" | cut -d'"' -f2 || true; }
DESC=$(grep '^description' "$MAN" | cut -d'"' -f2)
ARCH=$(dpkg --print-architecture)
TARGET=${CARGO_TARGET_DIR:-$OSM_ROOT/target}

log "compilando $APP (release)"
cargo build --release -p "$APP" --locked

STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
install -Dm755 "$TARGET/release/$APP" "$STAGE/usr/bin/$APP"
desktop=$(meta desktop)
[[ -n $desktop ]] && install -Dm644 "$OSM_ROOT/apps/$APP/$desktop" "$STAGE/usr/share/applications/$(basename "$desktop")"
install -Dm644 "$OSM_ROOT/LICENSE" "$STAGE/usr/share/doc/$APP/copyright"

mkdir -p "$STAGE/DEBIAN"
cat > "$STAGE/DEBIAN/control" <<CTL
Package: $APP
Version: $OSM_VERSION
Architecture: $ARCH
Maintainer: OSMarcio Linux <osmarcio-linux@localhost>
Section: $(meta section)
Priority: optional
Depends: $(meta depends)
Installed-Size: $(du -sk "$STAGE/usr" | cut -f1)
Description: $DESC
 Parte do OSMarcio Linux $OSM_VERSION.
CTL

OUTDIR="$OSM_ROOT/out/debs"; mkdir -p "$OUTDIR"
DEB="$OUTDIR/${APP}_${OSM_VERSION}_${ARCH}.deb"
dpkg-deb --root-owner-group --build "$STAGE" "$DEB" >/dev/null
log "pacote: ${DEB#$OSM_ROOT/}"
dpkg-deb --info "$DEB" | sed -n '/Package:/,$p' >&2
