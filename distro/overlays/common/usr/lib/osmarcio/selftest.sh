#!/bin/bash
# OSMarcio Linux — autoteste de integração executado DENTRO da VM.
#
# Só roda quando o hipervisor passa o sinal fw_cfg "opt/osmarcio/selftest"
# (ver scripts/test/test-boot-qemu.sh). Em uma instalação normal nunca roda.
# Cada verificação imprime uma linha "OSMTEST <PASS|FAIL> <nome> <detalhe>"
# no console serial; o script do host lê essas linhas como evidência.
set -u

FWCFG=/sys/firmware/qemu_fw_cfg/by_name/opt/osmarcio/selftest/raw
modprobe qemu_fw_cfg 2>/dev/null || true
[[ -r $FWCFG ]] || exit 0
MODE=$(tr -d '\0\n' < "$FWCFG")

out() { echo "OSMTEST $*" > /dev/ttyAMA0 2>/dev/null || echo "OSMTEST $*" > /dev/console; }
check() { # check <nome> <comando...>
  local name=$1; shift
  local res
  if res=$("$@" 2>&1); then out "PASS $name $(echo "$res" | tail -1 | cut -c1-120)"
  else out "FAIL $name $(echo "$res" | tail -1 | cut -c1-120)"; fi
}

out "BEGIN mode=$MODE"

# Persistência: contador de boots gravado em disco.
mkdir -p /var/lib/osmarcio
n=$(( $(cat /var/lib/osmarcio/boot-count 2>/dev/null || echo 0) + 1 ))
echo "$n" > /var/lib/osmarcio/boot-count; sync
out "INFO boot-count=$n"


check kernel        uname -r
check os-release    grep -q '^ID=osmarcio-linux' /etc/os-release
check systemd-pid1  bash -c '[[ $(cat /proc/1/comm) == systemd ]] && echo systemd'
check root-ext4     findmnt -no FSTYPE /
check efi-mounted   findmnt -no FSTYPE /boot/efi
check user-exists   id osmarcio
check root-locked   bash -c 'passwd -S root | grep -q " L " && echo locked'
check no-other-pw   bash -c 'x=$(awk -F: "\$2 !~ /^[!*]/ && \$1 != \"osmarcio\" {print \$1}" /etc/shadow); [[ -z $x ]] && echo "só osmarcio tem senha" || { echo "$x"; false; }'
check net-default-route bash -c 'ip -4 route show default | grep -q default && ip -4 route show default'
check net-ipv4      bash -c 'ip -4 -o addr show scope global | awk "{print \$2, \$4}" | grep .'
check dns           bash -c 'getent hosts deb.debian.org | head -1'
check https         curl -fsS -o /dev/null -w '%{http_code} %{ssl_verify_result}' https://deb.debian.org/debian/
check timesync      timedatectl show -p NTP --value
check apt-update    bash -c 'apt-get update -q 2>&1 | tail -1'
check apt-install   bash -c 'DEBIAN_FRONTEND=noninteractive apt-get install -y -q hello >/dev/null && hello'
check apt-remove    bash -c 'apt-get remove -y -q hello >/dev/null && ! command -v hello && echo removed'
check apt-signed    bash -c '! apt-config dump | grep -qi "AllowInsecureRepositories \"true\"" && n=$(ls /var/lib/apt/lists/*_InRelease | wc -l) && [[ $n -ge 3 ]] && echo "$n InRelease assinados verificados"'
check user-pw-temp  bash -c 'passwd -S osmarcio | grep -q " P " && [[ $(getent shadow osmarcio | cut -d: -f3) == 0 ]] && echo "senha temporária, troca obrigatória"'
check issue-notice  test -s /etc/issue.d/50-osmarcio-firstboot.issue
check no-failed-units bash -c 'f=$(systemctl --failed --no-legend --plain | awk "{print \$1}" | grep -v osmarcio-selftest); [[ -z $f ]] && echo nenhuma || { echo $f; false; }'
check rootfs-grown  bash -c 'df -h / | tail -1'

check grub-symlink  bash -c 'grep -q "linux  */vmlinuz " /boot/grub/grub.cfg && readlink /vmlinuz'

# Medidas reais de recursos (requisito: medir RAM/CPU/armazenamento).
out "INFO mem-used-MiB=$(LC_ALL=C free -m | awk '/^Mem:/{print $3}') mem-total-MiB=$(LC_ALL=C free -m | awk '/^Mem:/{print $2}')"
out "INFO root-used=$(df -h --output=used / | tail -1 | tr -d ' ') procs=$(ps -e --no-headers | wc -l)"
# systemd-analyze só responde depois do boot terminar (e este serviço faz
# parte do boot); /proc/uptime dá o tempo desde o início do kernel.
out "INFO uptime-at-selftest-end-s=$(cut -d' ' -f1 /proc/uptime) userspace-units=$(systemctl list-units --state=active --no-legend | wc -l)"
# Perfil desktop: verificações antes do login gráfico ---------------------------
if [[ -e /usr/sbin/lightdm ]]; then
  check default-target   bash -c '[[ $(systemctl get-default) == graphical.target ]] && echo graphical.target'
  check display-manager  bash -c 'for i in $(seq 60); do systemctl is-active -q lightdm && pgrep -x Xorg >/dev/null && pgrep -f lightdm-gtk-greeter >/dev/null && { echo "lightdm + Xorg + greeter ativos"; exit 0; }; sleep 2; done; systemctl is-active lightdm; false'
  check xorg-driver      bash -c 'grep -m1 -E "modeset\(0\): (Output|Modeline|glamor)" /var/log/Xorg.0.log | cut -c1-110'
  # Arquitetura esperada dos binários = a da máquina (ARM aarch64 ou x86-64).
  ELF=$(case $(uname -m) in aarch64) echo "ARM aarch64";; x86_64) echo "x86-64";; *) uname -m;; esac)
  check app-terminal     bash -c 'f=$(command -v osmarcio-terminal) && file -L "$f" | grep -o "'"$ELF"'" && ! ldd "$f" | grep "not found"'
  check app-info         bash -c 'osmarcio-info --version | head -1'
  check app-browser-pkg  bash -c 'dpkg-query -W -f="\${Package} \${Version} \${Architecture}" osmarcio-browser'
  check app-browser-arch bash -c 'file -L /usr/lib/osmarcio-browser/osmarcio-browser /usr/lib/osmarcio-browser/libcef.so | grep -c "'"$ELF"'" | grep -qx 2 && echo "osmarcio-browser + libcef.so: '"$ELF"'"'
  check app-browser-libs bash -c 'm=$(ldd /usr/lib/osmarcio-browser/libcef.so /usr/lib/osmarcio-browser/osmarcio-browser | grep "not found"); [[ -z $m ]] && echo "nenhuma biblioteca ausente" || { echo $m; false; }'
  check app-browser-version bash -c 'runuser -u osmarcio -- osmarcio-browser --version | tr -s " " | paste -sd"|" | cut -c1-118'
  check app-browser-icon bash -c 'ls /usr/share/icons/hicolor/{16x16,24x24,32x32,48x48,64x64,128x128,256x256}/apps/osmarcio-browser.png /usr/share/icons/hicolor/scalable/apps/osmarcio-browser.svg >/dev/null && grep -q "^Icon=osmarcio-browser$" /usr/share/applications/osmarcio-browser.desktop && echo "8 ícones + .desktop"'
  check userns-sandbox   bash -c 'v=$(cat /proc/sys/kernel/unprivileged_userns_clone 2>/dev/null || echo 1); [[ $v == 1 ]] && echo "user namespaces sem privilégio habilitados"'
fi

out "END"
if [[ $MODE == *desktop* ]]; then
  # A fase gráfica espera o login (feito pelo host) e depois desliga a VM.
  systemd-run --unit=osmarcio-selftest-desktop --collect /usr/lib/osmarcio/selftest-desktop.sh "$MODE"
  exit 0
fi
[[ $MODE == *poweroff* ]] && systemctl poweroff
exit 0
