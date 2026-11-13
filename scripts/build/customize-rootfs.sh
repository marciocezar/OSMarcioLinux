#!/usr/bin/env bash
# Hook de customização do mmdebstrap: aplica identidade e configuração
# do OSMarcio Linux dentro do rootfs recém-criado.
# uso: customize-rootfs.sh <rootdir> <arch> <perfil>
source "$(dirname "$0")/../lib/common.sh"
R=${1:?rootdir}; ARCH=${2:?arch}; PROFILE=${3:?perfil}

ch() { chroot "$R" "$@"; }

# Overlays: common + específico do perfil (se existir).
for ov in common "$PROFILE"; do
  d="$OSM_ROOT/distro/overlays/$ov"
  [[ -d $d ]] && cp -a "$d/." "$R/"
done
sed -i "s/@VERSION@/$OSM_VERSION/g" "$R/etc/os-release"
rm -f "$R/usr/lib/os-release"; ln -s ../../etc/os-release "$R/usr/lib/os-release"
chmod 0755 "$R"/usr/lib/osmarcio/*.sh

# Identidade de rede
echo osmarcio-linux > "$R/etc/hostname"
cat > "$R/etc/hosts" <<'EOF'
127.0.0.1   localhost
127.0.1.1   osmarcio-linux
::1         localhost ip6-localhost ip6-loopback
EOF

# Locale/fuso padrão (alteráveis depois pelo usuário).
echo "pt_BR.UTF-8 UTF-8" >  "$R/etc/locale.gen"
echo "en_US.UTF-8 UTF-8" >> "$R/etc/locale.gen"
ch locale-gen >/dev/null
echo 'LANG=pt_BR.UTF-8' > "$R/etc/default/locale"
ln -sf /usr/share/zoneinfo/America/Sao_Paulo "$R/etc/localtime"
echo America/Sao_Paulo > "$R/etc/timezone"

# Segurança: root bloqueado, sem senhas, machine-id vazio (gerado no 1º boot).
ch passwd -l root >/dev/null
: > "$R/etc/machine-id"
rm -f "$R/var/lib/dbus/machine-id"
rm -f "$R"/etc/ssh/ssh_host_* 2>/dev/null || true

# initramfs: módulos virtio garantidos (UTM/QEMU/VirtualBox).
cat >> "$R/etc/initramfs-tools/modules" <<'EOF'
virtio_pci
virtio_blk
virtio_scsi
virtio_net
EOF

# Serviços
# Rede: systemd-networkd/resolved em todos os perfis (o perfil desktop ainda
# não usa NetworkManager; o DHCP cabeado de 20-wired.network basta em VM).
ch systemctl enable systemd-networkd systemd-resolved >/dev/null 2>&1

# Perfil desktop: login gráfico + apps OSMarcio -------------------------------
if [[ $PROFILE != minimal ]]; then
  ARCH_DEBS="$OSM_ROOT/out/debs"
  for app in osmarcio-terminal osmarcio-info; do
    deb="$ARCH_DEBS/${app}_${OSM_VERSION}_${ARCH}.deb"
    [[ -f $deb ]] || die "pacote ausente: ${deb#$OSM_ROOT/} (rode 'make debs')"
    cp "$deb" "$R/tmp/"
    ch env DEBIAN_FRONTEND=noninteractive apt-get install -y -q --no-install-recommends "/tmp/$(basename "$deb")" >/dev/null
    rm -f "$R/tmp/$(basename "$deb")"
  done
  # OSMarcio Browser: bundle Rust + runtime CEF (scripts/build/browser-bundle.sh).
  BUNDLE="$OSM_ROOT/out/browser-bundle-$ARCH-v$OSM_VERSION"
  [[ -x $BUNDLE/osmarcio-browser && -f $BUNDLE/libcef.so ]] \
    || die "bundle do browser ausente: ${BUNDLE#$OSM_ROOT/} (rode 'make browser-bundle')"
  mkdir -p "$R/opt/osmarcio"
  cp -a "$BUNDLE" "$R/opt/osmarcio/browser"
  # Senha temporária do 1º boot também na tela de login gráfica (pam_echo
  # mostra o arquivo como mensagem informativa; some quando o arquivo some).
  sed -i '1a auth    optional        pam_echo.so file=/etc/issue.d/50-osmarcio-firstboot.issue' "$R/etc/pam.d/lightdm"
  ch systemctl set-default graphical.target >/dev/null
  ch systemctl enable lightdm >/dev/null 2>&1 || true
  [[ -L $R/etc/systemd/system/display-manager.service ]] || die "display-manager.service não configurado"
fi
ch systemctl enable systemd-timesyncd osmarcio-firstboot osmarcio-firstboot-cleanup osmarcio-selftest >/dev/null 2>&1
ln -sf ../run/systemd/resolve/stub-resolv.conf "$R/etc/resolv.conf"

# Regenera o initrd com a configuração acima.
ch update-initramfs -u -k all >/dev/null

# Manifesto de pacotes (para o manifesto de release).
mkdir -p "$R/usr/share/osmarcio"
ch dpkg-query -W -f='${Package}\t${Version}\t${Architecture}\n' > "$R/usr/share/osmarcio/packages.tsv"

# Limpeza: cache APT fora da imagem.
ch apt-get clean
rm -rf "$R"/var/lib/apt/lists/* "$R"/tmp/* "$R"/var/log/*.log
