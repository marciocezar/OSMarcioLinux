#!/bin/bash
# OSMarcio Linux — primeiro boot.
#  1. expande a partição raiz até o fim do disco virtual;
#  2. gera chaves/IDs únicos da máquina;
#  3. cria o usuário 'osmarcio' com SENHA ALEATÓRIA, de troca obrigatória.
# Nenhuma senha fixa é embutida na imagem.
set -euo pipefail

STAMP=/var/lib/osmarcio/firstboot.done
[[ -e $STAMP ]] && exit 0
mkdir -p /var/lib/osmarcio

say() { echo "osmarcio-firstboot: $*"; }

# 1. expandir raiz -------------------------------------------------------------
rootsrc=$(findmnt -no SOURCE /)
disk="/dev/$(lsblk -no PKNAME "$rootsrc")"
partnum=$(cat "/sys/class/block/$(basename "$rootsrc")/partition")
if growpart "$disk" "$partnum"; then
  resize2fs "$rootsrc" && say "partição raiz expandida"
else
  say "partição raiz já ocupa o disco inteiro"
fi

# 2. machine-id único -----------------------------------------------------------
if [[ ! -s /etc/machine-id ]]; then
  systemd-machine-id-setup
fi

# 3. usuário padrão --------------------------------------------------------------
USER_NAME=osmarcio
id "$USER_NAME" >/dev/null 2>&1 || useradd -m -s /bin/bash -G sudo -c "OSMarcio" "$USER_NAME"
# Só define a senha temporária se a conta ainda não tiver senha utilizável
# (estado L/NP): torna o passo idempotente caso um boot anterior falhe aqui.
if passwd -S "$USER_NAME" | awk '{exit !($2=="L" || $2=="NP")}'; then
  # Sem pipe "tr </dev/urandom | head": com pipefail o SIGPIPE abortaria o script.
  pw=$(head -c 1024 /dev/urandom | LC_ALL=C tr -dc 'A-HJ-NP-Za-km-z2-9')
  pw=${pw:0:12}
  [[ ${#pw} -eq 12 ]] || { say "falha ao gerar senha aleatória"; exit 1; }
  echo "$USER_NAME:$pw" | chpasswd
  chage -d 0 "$USER_NAME"          # força troca no primeiro login
  mkdir -p /etc/issue.d
  cat > /etc/issue.d/50-osmarcio-firstboot.issue <<EOF

  Primeiro boot do OSMarcio Linux
  usuário: $USER_NAME   senha temporária: $pw
  (a troca da senha é obrigatória no primeiro login; esta mensagem some depois)

EOF
  say "usuário '$USER_NAME' criado; senha temporária exibida na tela de login"
fi

touch "$STAMP"
