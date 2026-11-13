#!/bin/bash
# Remove a mensagem com a senha temporária assim que o usuário a trocar.
# /etc/shadow campo 3 == 0 significa "troca obrigatória ainda pendente".
set -eu
f=/etc/issue.d/50-osmarcio-firstboot.issue
[[ -e $f ]] || exit 0
lastchg=$(getent shadow osmarcio | cut -d: -f3)
if [[ -n "$lastchg" && "$lastchg" != "0" ]]; then
  rm -f "$f"
fi
