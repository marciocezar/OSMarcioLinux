# Rede — configuração e diagnóstico

## Como está configurado (perfil Minimal)

| Função | Componente | Arquivo |
|---|---|---|
| Endereço IP (DHCP) | `systemd-networkd` | `/etc/systemd/network/20-wired.network` (casa `en*`/`eth*`) |
| DNS | `systemd-resolved` | `/etc/resolv.conf` → `/run/systemd/resolve/stub-resolv.conf` |
| Hora (NTP) | `systemd-timesyncd` | padrão Debian |
| TLS | OpenSSL 3.5 + `ca-certificates` | `/etc/ssl/certs` |
| Firewall | `nftables` instalado, **sem regras ativas** (pendente) | — |

Os perfis desktop/developer usarão **NetworkManager** (Wi-Fi, VPN, applet).

No UTM/QEMU com NAT ("user networking") a VM recebe `10.0.2.15/24`,
gateway `10.0.2.2` — valores registrados no autoteste.

## Diagnóstico passo a passo

```bash
ip -br link                 # a interface existe? (ex.: enp0s1 UP)
ip -br addr                 # tem IPv4? (ex.: 10.0.2.15/24)
ip route                    # rota padrão: "default via 10.0.2.2"
networkctl status           # visão do systemd-networkd
resolvectl status           # servidores DNS em uso
resolvectl query deb.debian.org
ping -c3 10.0.2.2           # gateway (o NAT do QEMU pode não responder ICMP para fora)
curl -I https://deb.debian.org   # TCP + TLS + HTTP
ss -tunap                   # sockets abertos
timedatectl                 # "System clock synchronized: yes"
```

`nmcli` e `wget` também funcionam (`wget` está no Minimal; `nmcli` virá
com o NetworkManager no perfil desktop).

## Problemas comuns

| Sintoma | Verifique |
|---|---|
| Sem IPv4 | `journalctl -u systemd-networkd`; nome da interface casa com `en*`? |
| DNS falha, IP funciona | `ls -l /etc/resolv.conf` deve apontar para o stub do resolved |
| HTTPS falha com erro de certificado | relógio errado (`timedatectl`) — TLS depende da data |
| `apt update` lento | IPv6 sem rota; teste `curl -4 -I https://deb.debian.org` |
