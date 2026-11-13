#!/usr/bin/env python3
"""Cliente mínimo do QEMU Guest Agent (QGA) por socket unix.

uso:
  qga.py <socket> exec <comando shell>      executa como root na VM; imprime saída; sai com o código do comando
  qga.py <socket> push <local> <remoto>     copia arquivo do host para a VM
  qga.py <socket> pull <remoto> <local>     copia arquivo da VM para o host

Usado nos testes do desktop/browser para inspecionar a VM e instalar um
build novo do app sem reconstruir a imagem inteira. Só fala com a VM de
teste local (socket criado por scripts/run/qemu-arm64.sh --qga).
"""
import base64
import json
import socket
import sys
import time


class Qga:
    def __init__(self, path):
        self.s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.s.connect(path)
        self.f = self.s.makefile("rwb")
        # guest-sync descarta respostas antigas pendentes no canal.
        tok = int(time.time()) & 0xFFFFFF
        self.call("guest-sync", {"id": tok}, sync=tok)

    def call(self, cmd, args=None, sync=None):
        msg = {"execute": cmd}
        if args is not None:
            msg["arguments"] = args
        self.f.write(json.dumps(msg).encode() + b"\n")
        self.f.flush()
        while True:
            line = self.f.readline()
            if not line:
                raise SystemExit("qga: conexão encerrada")
            line = line.lstrip(b"\xff").strip()
            if not line:
                continue
            r = json.loads(line)
            if "error" in r:
                raise SystemExit(f"qga: {cmd}: {r['error'].get('desc')}")
            if sync is not None and r.get("return") != sync:
                continue
            return r.get("return")

    def exec(self, shell, timeout=300):
        pid = self.call("guest-exec", {"path": "/bin/sh", "arg": ["-c", shell], "capture-output": True})["pid"]
        t0 = time.time()
        while True:
            st = self.call("guest-exec-status", {"pid": pid})
            if st.get("exited"):
                out = base64.b64decode(st.get("out-data", "")) + base64.b64decode(st.get("err-data", ""))
                return st.get("exitcode", 1), out.decode(errors="replace")
            if time.time() - t0 > timeout:
                raise SystemExit("qga: timeout")
            time.sleep(0.3)

    def push(self, local, remote):
        h = self.call("guest-file-open", {"path": remote, "mode": "wb"})
        with open(local, "rb") as fh:
            while chunk := fh.read(48 * 1024):
                self.call("guest-file-write", {"handle": h, "buf-b64": base64.b64encode(chunk).decode()})
        self.call("guest-file-close", {"handle": h})

    def pull(self, remote, local):
        h = self.call("guest-file-open", {"path": remote, "mode": "rb"})
        with open(local, "wb") as fh:
            while True:
                r = self.call("guest-file-read", {"handle": h, "count": 48 * 1024})
                fh.write(base64.b64decode(r["buf-b64"]))
                if r.get("eof"):
                    break
        self.call("guest-file-close", {"handle": h})


def main():
    if len(sys.argv) < 4:
        raise SystemExit(__doc__)
    q = Qga(sys.argv[1])
    op = sys.argv[2]
    if op == "exec":
        code, out = q.exec(" ".join(sys.argv[3:]))
        sys.stdout.write(out)
        sys.exit(code)
    elif op == "push":
        q.push(sys.argv[3], sys.argv[4])
    elif op == "pull":
        q.pull(sys.argv[3], sys.argv[4])
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main()
