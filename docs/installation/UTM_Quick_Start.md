# UTM Quick Start — OSMarcio Linux Desktop (Mac Apple Silicon)

Guia curto e específico para a imagem
**`images/utm/OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0.img`**.

| | |
|---|---|
| Tipo | **disco RAW pronto** (GPT + UEFI), **não é ISO** |
| Arquitetura | ARM64 / AArch64 |
| Firmware | UEFI (edk2 do UTM) — GRUB em `EFI/BOOT/BOOTAA64.EFI` |
| Partições | 1: ESP FAT32 256 MiB · 2: raiz ext4 (expande sozinha no 1º boot) |
| Sistema | Debian 13 + Linux 6.12 LTS + systemd |
| Desktop | LightDM → XFCE (X11) |
| Apps | OSMarcio Terminal, OSMarcio Browser (Rust + CEF), `osmarcio-info` |

> **Erro mais comum (foi o erro original):** anexar a `.img` como
> **CD/DVD / "Boot ISO Image"**. O firmware não acha nada para iniciar e cai
> no `Shell>` do UEFI (ou numa tela preta). A `.img` precisa ser um
> **disco VirtIO importado**. Ver [relatório](../../reports/OSMarcioLinux_UTM_Boot_Fix.md).

---

## Caminho rápido (recomendado): `make run-utm`

```bash
make doctor-mac
```

```bash
make run-utm
```

Na 1ª vez o comando:

1. converte a IMG para `out/utm/OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0.qcow2`
   e aumenta o disco para 20 GiB (a IMG original não é alterada);
2. cria no UTM a VM **"OSMarcio Linux 0.2.0 Desktop"**: QEMU, ARM64,
   virtualização (HVF), UEFI, 4 GiB, 4 CPUs, disco **VirtIO**, rede
   compartilhada `virtio-net-pci`, vídeo `virtio-gpu-pci`;
3. inicia a VM.

Nas próximas vezes ele só **inicia** a VM existente — nunca recria nem
sobrescreve o disco. Para estado da VM: `/Applications/UTM.app/Contents/MacOS/utmctl list`.

---

## Caminho manual (pela interface do UTM)

1. **Qual imagem:** `images/utm/OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0.img`.
   Confira antes: `make verify-images`. Copie-a para fora do repositório
   (ex.: `~/UTM-OSMarcio-Desktop.img`) — o UTM pode modificar o arquivo importado.
2. **Arquitetura:** UTM → **+** → **Virtualize** (não *Emulate*) → **Linux**.
   *Virtualize* = ARM64 nativo via Hypervisor.framework (rápido).
   *Emulate* só seria necessário para uma imagem x86_64, que **não** é o caso.
3. **Virtualização ou emulação:** virtualização. Deixe **desmarcado**
   "Use Apple Virtualization" (o backend deve ser **QEMU**).
4. **Boot image:** **não** selecione ISO. Deixe o campo *Boot ISO Image* vazio → Continue.
5. **CPU e RAM:** 4096 MiB (mínimo prático 2048), 4 núcleos → Continue.
6. **Disco:** qualquer tamanho (será removido) → Continue → Save, **sem iniciar**.
   Abra **Edit** da VM → **Drives**: apague o disco criado → **New Drive** →
   **Import…** → escolha a `.img` → interface **VirtIO** → Save.
   Depois, no drive importado, **Resize…** para 20 GB (o 1º boot expande a raiz).
   Confirme que **não há** nenhum drive "CD/DVD" ou "USB" com a imagem.
7. **Rede:** *Network* → **Shared Network**, placa **virtio-net-pci**.
8. **Vídeo:** *Display* → **virtio-gpu-pci** (sem "-gl"), *Resize display to
   window size automatically* ligado (o `spice-vdagent` da imagem ajusta a resolução).
9. **System:** arquitetura ARM64 (aarch64), máquina `virt`, **UEFI Boot** ligado
   (padrão). Salve e clique **▶︎**.

## Primeiro boot

10. **Login:** o GRUB espera 3 s; em ~10–20 s aparece a tela do **LightDM**
    com o usuário **OSMarcio** e um aviso azul com a **senha temporária**
    (aleatória, gerada no 1º boot). Digite-a e **Entrar**.
    O sistema pede imediatamente a troca de senha:
    *Changing password* → senha temporária de novo → nova senha → repetir a nova.
11. **Desktop:** abre o XFCE: painel inferior com menu **OSMarcio**,
    lançadores do Terminal e do Browser, lista de janelas, relógio e
    botão de sessão (sair/reiniciar/desligar).
12. **Terminal:** clique no ícone de terminal do painel (2º ícone, ao lado do
    menu) ou menu **OSMarcio → Emulador de terminal**. Teste: `osmarcio-info --all`.
13. **Browser:** ícone de globo do painel (3º ícone), menu **OSMarcio →
    Navegador web**, ou no terminal `osmarcio-browser https://example.com`.
    > Os lançadores do painel e as entradas do menu estão configurados, mas o
    > clique neles **não** foi testado automaticamente — os testes abrem os
    > apps por comando (`osmarcio-terminal`, `osmarcio-browser`).
14. **Logs:**
    ```bash
    journalctl -b -p warning
    journalctl -b -u lightdm
    cat /var/log/Xorg.0.log | grep -E '\(EE\)|modeset'
    cat ~/.xsession-errors
    cat ~/.cache/osmarcio/browser/cef.log
    RUST_LOG=info osmarcio-browser https://example.com
    ```
    No host, sem abrir a janela: `utmctl ip-address "OSMarcio Linux 0.2.0 Desktop"`
    e `utmctl exec …` (usa o `qemu-guest-agent` da imagem).

## 15. Erros comuns

| Sintoma | Causa | Solução |
|---|---|---|
| `Shell>` do UEFI / "BdsDxe: failed to load Boot0001 … Not Found" | IMG anexada como CD/USB/ISO, ou disco VirtIO vazio | importe a IMG como disco **VirtIO** (passo 6) e remova o CD |
| "Image type X64 can't be loaded on AARCH64" no log | é a ROM PXE x86 da placa de rede — inofensivo | nada; se o boot falha é pelo item acima |
| Tela preta logo após o GRUB | vídeo `virtio-ramfb`/`-gl` ou VM criada em *Emulate* x86_64 | use `virtio-gpu-pci` e *Virtualize* ARM64 |
| LightDM não aparece, só login em texto | sem placa de vídeo (Display removido) | adicione Display `virtio-gpu-pci`; `systemctl status lightdm` |
| Senha temporária não aparece | não é o 1º boot (já foi trocada) | use a sua senha; o aviso some depois da troca |
| Resolução fixa | redimensionamento dinâmico desligado (não testado nesta versão) | Display → ligar *Resize display…*; `systemctl status spice-vdagentd`; ou Configurações → Monitor |
| Browser abre mas não carrega | sem rede | `ip -br a`, `resolvectl status`, `curl -I https://example.com` |
| Disco cheio / só 1,5 GB | IMG importada sem Resize | Resize no UTM; reinicie (o 1º boot expande; depois `sudo growpart /dev/vda 2 && sudo resize2fs /dev/vda2`) |

## Sem UTM (QEMU com janela, mesmo hardware virtual)

```bash
make run-qemu-arm64
```

Testes automatizados (QEMU/HVF): `make test-boot` (boot + console) e
`make test-desktop` (login gráfico real, XFCE, Terminal, Browser, capturas
de tela em `reports/tests/screenshots/`).
