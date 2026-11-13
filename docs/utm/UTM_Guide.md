# Guia — OSMarcio Linux no UTM (Mac Apple Silicon)

> **Imagem desktop (0.2.0):** use o guia curto
> [docs/installation/UTM_Quick_Start.md](../installation/UTM_Quick_Start.md)
> e `make run-utm`. Este guia descreve a imagem **Minimal** (só console).
> Atenção: a `.img` é um **disco**; anexá-la como CD/ISO faz o UEFI cair no `Shell>`.

Imagem: `images/utm/OSMarcioLinux-UTM-AArch64-Minimal-v0.1.0.img`
(RAW, GPT, UEFI, ARM64). Ela **não** é uma ISO de instalação: é um disco
pronto, como o de uma VM recém-instalada.

> Testado: boot no QEMU 11 com HVF (o mesmo motor do UTM), via
> `make test-utm`. A importação no **aplicativo** UTM segue os passos
> abaixo, mas **ainda não foi verificada** com uma execução registrada.

## 0. Gerar e verificar a imagem

```bash
make image-utm
```

```bash
make verify-images
```

## 1. Teste rápido sem UTM (QEMU com janela)

```bash
make run-qemu-arm64
```

Isso cria **uma vez** `out/vm/<imagem>.qcow2` (20 GiB virtuais) a partir da
imagem — a imagem original nunca é alterada e um disco existente nunca é
sobrescrito. O console serial aparece no terminal; a janela mostra o tty1.

## 2. Criar a VM no UTM

1. **Copie** a imagem para um lugar seu (o UTM pode modificar o arquivo):
   `cp images/utm/OSMarcioLinux-UTM-AArch64-Minimal-v0.1.0.img ~/UTM-OSMarcio.img`
2. UTM → **Create a New Virtual Machine** → **Virtualize** → **Linux**.
3. Desmarque "Use Apple Virtualization"; deixe **QEMU**. Em *Boot image*
   não selecione ISO → **Continue**.
4. Hardware: 4096 MiB de RAM, 4 núcleos → **Continue**.
5. Storage: qualquer tamanho (o disco criado será removido no passo 7).
6. Finalize, **sem iniciar**. Abra **Edit** da VM.
7. **Drives**: remova o disco criado; **New Drive → Import…** → escolha
   `~/UTM-OSMarcio.img`, interface **VirtIO**. O UTM converte para qcow2.
8. (Opcional) **Resize** o disco importado (ex.: 20 GB) — o 1º boot
   expande a partição raiz automaticamente.
9. **System**: arquitetura ARM64 (aarch64), máquina `virt`; **UEFI Boot**
   ligado (padrão).
10. **Network**: *Shared Network* (NAT), placa **virtio-net-pci**.
11. **Display**: `virtio-gpu-pci` (ou `virtio-ramfb`). **Serial**: opcional.

## 3. Primeiro boot

- O GRUB mostra o menu por 3 s; depois o kernel e o systemd sobem.
- O serviço `osmarcio-firstboot` expande o disco e cria o usuário
  `osmarcio` com **senha temporária aleatória**, exibida na tela de login.
- Faça login; o sistema **exige** trocar a senha. A mensagem com a senha
  temporária desaparece no boot seguinte.

## 4. Verificações dentro da VM

```bash
cat /etc/os-release
ip -br addr && ip route
resolvectl status | head
curl -I https://deb.debian.org
sudo apt update && sudo apt install -y htop
```

## 5. Atualizar a imagem sem perder dados

Uma nova imagem **não** deve substituir o disco da sua VM — isso apagaria
seus arquivos. Dentro da VM, atualize por APT (`sudo apt full-upgrade`).
Para testar uma versão nova da imagem, crie **outra** VM.

## Problemas comuns

| Sintoma | Causa / solução |
|---|---|
| Tela preta, nada no serial | UEFI desligado, ou disco importado como IDE/NVMe sem driver — use VirtIO |
| `Shell>` do UEFI | o disco não foi anexado; confira *Drives* |
| Sem rede | placa diferente de virtio-net ou rede "Host Only" |
| Senha temporária não aparece | role a tela (`Shift+PgUp`) ou veja `sudo journalctl -u osmarcio-firstboot` pelo modo de recuperação |
