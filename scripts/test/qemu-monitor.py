#!/usr/bin/env python3
"""Controle mínimo do monitor HMP do QEMU por socket unix.

uso:
  qemu-monitor.py <socket> screendump <arquivo.png>
  qemu-monitor.py <socket> type <texto>        (letras, dígitos e .-_/ espaço)
  qemu-monitor.py <socket> key <tecla-qemu>... (ex.: ret, tab, ctrl-alt-f1)
  qemu-monitor.py <socket> cmd <comando HMP>
  qemu-monitor.py - pngstat <arquivo.png> x0 y0 x1 y1
      fração de pixels claros (luma > 200) e nº de cores na região — usado
      para provar que a página web foi pintada (e não só a moldura do app)

Usado por scripts/test/test-desktop-qemu-arm64.sh para digitar a senha na
tela de login gráfica e capturar a tela — exatamente como um usuário faria.
"""
import socket
import struct
import sys
import time
import zlib

SPECIAL = {" ": "spc", ".": "dot", "-": "minus", "_": "shift-minus",
           "/": "slash", "\n": "ret", "=": "equal", ",": "comma",
           ";": "semicolon", ":": "shift-semicolon", "'": "apostrophe",
           '"': "shift-apostrophe", "|": "shift-backslash", "\\": "backslash",
           ">": "shift-dot", "<": "shift-comma", "$": "shift-4", "%": "shift-5",
           "&": "shift-7", "*": "shift-8", "(": "shift-9", ")": "shift-0",
           "@": "shift-2", "!": "shift-1", "#": "shift-3", "+": "shift-equal",
           "~": "shift-grave_accent", "[": "bracket_left", "]": "bracket_right",
           "{": "shift-bracket_left", "}": "shift-bracket_right", "?": "shift-slash", "^": "shift-6", "`": "grave_accent"}


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


def png_rgb(path):
    """Decodifica PNG RGB/RGBA 8 bits sem entrelaçamento (formato do screendump)."""
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "não é PNG"
    pos, idat, w = 8, b"", 0
    while pos < len(data):
        ln, typ = struct.unpack(">I4s", data[pos:pos + 8])
        chunk = data[pos + 8:pos + 8 + ln]
        if typ == b"IHDR":
            w, h, depth, ctype, _, _, inter = struct.unpack(">IIBBBBB", chunk)
            assert depth == 8 and ctype in (2, 6) and inter == 0, "PNG não suportado"
            bpp = 3 if ctype == 2 else 4
        elif typ == b"IDAT":
            idat += chunk
        pos += 12 + ln
    raw, stride, rows, prev = zlib.decompress(idat), w * bpp, [], bytearray(w * bpp)
    for y in range(h):
        f, line = raw[y * (stride + 1)], bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b, c = prev[i], prev[i - bpp] if i >= bpp else 0
            if f == 1: line[i] = (line[i] + a) & 255
            elif f == 2: line[i] = (line[i] + b) & 255
            elif f == 3: line[i] = (line[i] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c; pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(line); prev = line
    return w, h, bpp, rows


def pngstat(path, x0, y0, x1, y1):
    w, h, bpp, rows = png_rgb(path)
    light = total = 0; colors = set()
    for y in range(y0, min(y1, h)):
        r = rows[y]
        for x in range(x0, min(x1, w)):
            R, G, B = r[x * bpp], r[x * bpp + 1], r[x * bpp + 2]
            total += 1; colors.add((R, G, B))
            if 0.299 * R + 0.587 * G + 0.114 * B > 200: light += 1
    print(f"light={light / max(total, 1):.3f} colors={len(colors)} pixels={total}")


def main():
    if len(sys.argv) >= 3 and sys.argv[2] == "pngstat":
        pngstat(sys.argv[3], *map(int, sys.argv[4:8])); return
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
