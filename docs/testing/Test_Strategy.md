# Estratégia de testes

Princípio: **nada é declarado funcionando sem um teste que rode de verdade**
e deixe evidência em `reports/tests/`. Compilar não é testar; criar a ISO
não é testar a ISO.

## Níveis

| Nível | O que | Onde | Comando | Situação |
|---|---|---|---|---|
| Unitário | lógica pura (`osmarcio-core`, configs) | container (ou host) | `make apps-test` | ✅ 12 testes |
| Estático | rustfmt, clippy `-D warnings` | container | `make apps-lint` | ✅ |
| GUI | app real em X virtual (Xvfb + xdotool), verificando efeitos observáveis | container | `make terminal-test` | ✅ 10/10 |
| Pacote | `.deb` instala via APT em Debian 13 limpo, libs resolvem, remove | container `debian:trixie` | manual (ver Build Report) | ✅ |
| Boot/integração | imagem real, UEFI→GRUB→kernel→systemd, 2 boots, autoteste interno | QEMU + HVF no host | `make test-utm` | ✅ 22 verificações × 2 boots |
| Plataforma | importação e boot no app UTM | UTM | manual | ⏳ |
| Instalação | ISO → instalador → VDI → reboot | VirtualBox | `make test-install` | ⛔ Fase 15 |
| Hardware | RK3399 físico | ROCK Pi 4A | manual + log serial | ⛔ Fase 14 |

## O autoteste de boot

`distro/overlays/common/usr/lib/osmarcio/selftest.sh` roda **dentro** da VM
somente quando o QEMU passa `-fw_cfg name=opt/osmarcio/selftest` —
em instalação real o serviço sai sem fazer nada. Ele imprime no serial
linhas `OSMTEST PASS|FAIL <nome> <detalhe>`; o script do host
(`scripts/test/test-boot-qemu-arm64.sh`) coleta, conta falhas, gera o
relatório e anota o resultado no manifesto da imagem.

Verificações: kernel, os-release, systemd como PID 1, raiz ext4, ESP
montada, usuário criado, root bloqueado, nenhuma outra conta com senha,
senha temporária com troca obrigatória, aviso de login, rota padrão, IPv4,
DNS, HTTPS com TLS verificado, NTP, `apt update`, `apt install hello`,
`apt remove`, repositórios assinados, nenhuma unidade com falha, raiz
expandida, GRUB usando `/vmlinuz`; e medidas de RAM/processos/disco.

A imagem testada **nunca é alterada**: o teste usa um overlay qcow2
descartável. O 2º boot usa o mesmo overlay e prova persistência
(`boot-count` 1 → 2).

## Bugs reais encontrados pelos testes (até agora)

1. `firstboot.sh`: `tr </dev/urandom | head` + `pipefail` → SIGPIPE abortava
   o script e o usuário ficava **sem senha**. Corrigido; o passo ficou
   idempotente. (relatório `boot-arm64-20261113T021412Z.md`, 2 falhas)
2. Terminal: o `Rc` do estado da janela morria logo após a criação; nenhum
   atalho funcionava e `exit` não fechava a janela. Corrigido.
3. Medidas de RAM vazias por causa do locale pt_BR (`Mem.:`). Corrigido.
