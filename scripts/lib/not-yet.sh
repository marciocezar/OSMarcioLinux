#!/usr/bin/env bash
# Alvo do Makefile ainda não implementado: falha com mensagem clara (nunca finge sucesso).
# uso: not-yet.sh <alvo> <fase> [motivo]
printf '\033[1;31m[indisponível]\033[0m make %s — pertence à %s e ainda não foi implementado.\n' "$1" "$2" >&2
[[ -n ${3:-} ]] && printf '  motivo: %s\n' "$3" >&2
printf '  situação atual: veja PROGRESS.md\n' >&2
exit 2
