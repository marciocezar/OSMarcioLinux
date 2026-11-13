#!/usr/bin/env python3
"""Controle mínimo do monitor HMP do QEMU por socket unix.

uso:
  qemu-monitor.py <socket> screendump <arquivo.png>
  qemu-monitor.py <socket> type <texto>        (letras, dígitos e .-_/ espaço)
  qemu-monitor.py <socket> key <tecla-qemu>... (ex.: ret, tab, ctrl-alt-f1)
  qemu-monitor.py <socket> cmd <comando HMP>

Usado por scripts/test/test-desktop-qemu-arm64.sh para digitar a senha na
tela de login gráfica e capturar a tela — exatamente como um usuário faria.
"""
import socket
import sys
import time

SPECIAL = {" ": "spc", ".": "dot", "-": "minus", "_": "shift-minus",
           "/": "slash", "\n": "ret", "=": "equal", ",": "comma"}


def keyname(ch):
    if ch.isdigit() or ("a" <= ch <= "z"):
        return ch
    if "A" <= ch <= "Z":
        return "shift-" + ch.lower()
    if ch in SPECIAL:
        return SPECIAL[ch]
    raise SystemExit(f"caractere não suportado: {ch!r}")


def hmp(sock_path, commands, delay=0.08):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(sock_path)
    s.settimeout(2)
    try:
        s.recv(4096)  # banner
    except socket.timeout:
        pass
    for c in commands:
        s.sendall((c + "\n").encode())
        time.sleep(delay)
    time.sleep(0.5)
    s.settimeout(1)
    try:
        while s.recv(65536):
            pass
    except socket.timeout:
        pass
    s.close()


def main():
    if len(sys.argv) < 4:
        raise SystemExit(__doc__)
    sock, op, args = sys.argv[1], sys.argv[2], sys.argv[3:]
    if op == "screendump":
        hmp(sock, [f"screendump {args[0]} -f png"])
        time.sleep(1)
    elif op == "type":
        hmp(sock, [f"sendkey {keyname(c)}" for c in " ".join(args)])
    elif op == "key":
        hmp(sock, [f"sendkey {k}" for k in args], delay=0.3)
    elif op == "cmd":
        hmp(sock, [" ".join(args)])
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main()
