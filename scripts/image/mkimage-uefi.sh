#!/usr/bin/env bash
# Monta uma imagem de disco RAW/GPT inicializável por UEFI a partir de um rootfs.
# Roda DENTRO do container de build. Não usa loop devices nem mount:
#   - a ESP (FAT32) é criada num arquivo com mkfs.vfat + mtools;
#   - a raiz (ext4) é criada num arquivo com "mkfs.ext4 -d <dir>";
#   - as duas são copiadas para dentro de um disco GPT com sgdisk + dd.
# Portanto é impossível este script formatar um disco físico.
#
# uso: mkimage-uefi.sh <arm64|amd64> <rootfs.tar> <saida.img> [tamanho-raiz-MiB]
source "$(dirname "$0")/../lib/common.sh"

ARCH=${1:?arch}; TAR=${2:?rootfs.tar}; OUT=${3:?saida.img}; ROOT_MIB=${4:-}
ESP_MIB=256

case $ARCH in
  arm64) EFI_NAME=BOOTAA64.EFI; GRUB_FMT=arm64-efi; SERIAL=ttyAMA0 ;;
  amd64) EFI_NAME=BOOTX64.EFI;  GRUB_FMT=x86_64-efi; SERIAL=ttyS0 ;;
  *) die "arch inválida: $ARCH" ;;
esac

WORK=$(mktemp -d /tmp/osm-img.XXXX)
trap 'rm -rf "$WORK"' EXIT
R=$WORK/root
mkdir -p "$R"
log "extraindo rootfs"
tar --numeric-owner --xattrs --xattrs-include='*' -xpf "$TAR" -C "$R"

ROOT_UUID=$(uuidgen)
ESP_ID=$(printf '%08X' $((RANDOM * 65536 + RANDOM)))
ESP_UUID="${ESP_ID:0:4}-${ESP_ID:4:4}"
KVER=$(basename "$(ls -d "$R"/lib/modules/* | sort -V | tail -1)")
log "kernel: $KVER"

# fstab ----------------------------------------------------------------------
cat > "$R/etc/fstab" <<EOF
# OSMarcio Linux
UUID=$ROOT_UUID  /          ext4  errors=remount-ro  0 1
UUID=$ESP_UUID   /boot/efi  vfat  umask=0077         0 2
EOF
mkdir -p "$R/boot/efi"

# GRUB ------------------------------------------------------------------------
# A ESP contém só um GRUB "standalone" na rota de mídia removível
# (EFI/BOOT/BOOTxx.EFI), que não depende de variáveis NVRAM — essencial para
# UTM/QEMU/VirtualBox, onde a NVRAM pode ser descartada.
mkdir -p "$R/boot/grub"
cp -a "$R/usr/lib/grub/$GRUB_FMT" "$R/boot/grub/"
# O Debian (linux-base) mantém /vmlinuz e /initrd.img apontando para o kernel
# mais novo a cada instalação de kernel: usando-os, um "apt full-upgrade" que
# traga kernel novo continua bootando sem regenerar este arquivo.
if [[ -L $R/vmlinuz && -L $R/initrd.img ]]; then
  KPATH=/vmlinuz; IPATH=/initrd.img
else
  warn "symlinks /vmlinuz ausentes; usando caminho fixo do kernel $KVER"
  KPATH=/boot/vmlinuz-$KVER; IPATH=/boot/initrd.img-$KVER
fi
OLD_ENTRY=""
if [[ -L $R/vmlinuz.old && -L $R/initrd.img.old ]]; then
  OLD_ENTRY="menuentry \"OSMarcio Linux $OSM_VERSION (kernel anterior)\" {
    search --no-floppy --fs-uuid --set=root $ROOT_UUID
    linux  /vmlinuz.old root=UUID=$ROOT_UUID ro console=tty0 console=$SERIAL,115200
    initrd /initrd.img.old
}"
fi
cat > "$R/boot/grub/grub.cfg" <<EOF
# Gerado por scripts/image/mkimage-uefi.sh — OSMarcio Linux $OSM_VERSION
set timeout=3
set default=0
serial --unit=0 --speed=115200
terminal_input console serial
terminal_output gfxterm serial
insmod all_video
insmod gfxterm

menuentry "OSMarcio Linux $OSM_VERSION" {
    search --no-floppy --fs-uuid --set=root $ROOT_UUID
    linux  $KPATH root=UUID=$ROOT_UUID ro console=tty0 console=$SERIAL,115200
    initrd $IPATH
}
menuentry "OSMarcio Linux $OSM_VERSION (modo de recuperação)" {
    search --no-floppy --fs-uuid --set=root $ROOT_UUID
    linux  $KPATH root=UUID=$ROOT_UUID ro single console=tty0 console=$SERIAL,115200
    initrd $IPATH
}
$OLD_ENTRY
EOF

cat > "$WORK/early.cfg" <<EOF
search --no-floppy --fs-uuid --set=root $ROOT_UUID
set prefix=(\$root)/boot/grub
configfile /boot/grub/grub.cfg
EOF
cp "$WORK/early.cfg" "$R/tmp/early.cfg"
chroot "$R" grub-mkimage -O "$GRUB_FMT" -o /tmp/$EFI_NAME -c /tmp/early.cfg -p /boot/grub \
  part_gpt part_msdos fat ext2 normal linux configfile search search_fs_uuid \
  echo ls test gzio all_video efi_gop gfxterm font serial terminal reboot halt
mv "$R/tmp/$EFI_NAME" "$WORK/$EFI_NAME"; rm -f "$R/tmp/early.cfg"

# ESP (FAT32) -------------------------------------------------------------------
ESP=$WORK/esp.img
mkfs.vfat -C -F 32 -n OSM-EFI -i "$ESP_ID" "$ESP" $((ESP_MIB * 1024)) >/dev/null
mmd   -i "$ESP" ::/EFI ::/EFI/BOOT
mcopy -i "$ESP" "$WORK/$EFI_NAME" ::/EFI/BOOT/$EFI_NAME

# Raiz (ext4) ----------------------------------------------------------------------
used_mib=$(du -sm "$R" | cut -f1)
[[ -n $ROOT_MIB ]] || ROOT_MIB=$(( used_mib * 13 / 10 + 512 ))
log "raiz: ${used_mib} MiB usados, partição de ${ROOT_MIB} MiB (expande no 1º boot)"
ROOTIMG=$WORK/root.img
mkfs.ext4 -q -F -L osmarcio-root -U "$ROOT_UUID" -d "$R" "$ROOTIMG" "${ROOT_MIB}M"

# Disco GPT -------------------------------------------------------------------------
TOTAL_MIB=$(( 1 + ESP_MIB + ROOT_MIB + 1 ))
rm -f "$OUT.partial"
truncate -s "${TOTAL_MIB}M" "$OUT.partial"
sgdisk -Z "$OUT.partial" >/dev/null
sgdisk \
  -n 1:1M:+${ESP_MIB}M -t 1:EF00 -c 1:"EFI System" \
  -n 2:0:+${ROOT_MIB}M  -t 2:$([[ $ARCH == arm64 ]] && echo B921 || echo 8304) -c 2:"OSMarcio Linux root" \
  "$OUT.partial" >/dev/null
dd if="$ESP"     of="$OUT.partial" bs=1M seek=1 conv=notrunc,sparse status=none
dd if="$ROOTIMG" of="$OUT.partial" bs=1M seek=$((1 + ESP_MIB)) conv=notrunc,sparse status=none
sgdisk -v "$OUT.partial" | tail -1 >&2
mv "$OUT.partial" "$OUT"

# Metadados para o manifesto ----------------------------------------------------------
cat > "$OUT.build-info" <<EOF
kernel=$KVER
root_uuid=$ROOT_UUID
esp_uuid=$ESP_UUID
bootloader=GRUB $(chroot "$R" dpkg-query -W -f='${Version}' grub-common 2>/dev/null) ($GRUB_FMT, caminho removível EFI/BOOT/$EFI_NAME)
systemd=$(chroot "$R" dpkg-query -W -f='${Version}' systemd)
packages=$(wc -l < "$R/usr/share/osmarcio/packages.tsv")
EOF
cp "$R/usr/share/osmarcio/packages.tsv" "$OUT.packages.tsv"
log "imagem pronta: $OUT (${TOTAL_MIB} MiB)"
