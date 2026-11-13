# OSMarcio Linux — Correção do boot no UTM e validação da interface gráfica

- Data: 2026-11-13 · Host: macOS (Darwin 27.2) Apple Silicon, 10 CPUs, 24 GiB · UTM 4.7.5 · QEMU 11.0.1 (Homebrew)
- Versão da distribuição: 0.2.0 (`VERSION`)

## 1. Arquivo analisado

Não existe nenhuma **ISO** no projeto (`images/virtualbox/`, `images/releases/`,
`images/rockpi/` estão vazios; `*.iso` não foi encontrado em `~`). Os
artefatos existentes eram todos **ARM64**:

| Arquivo | Tipo |
|---|---|
| `images/utm/OSMarcioLinux-UTM-AArch64-Minimal-v0.1.0.img` | disco RAW GPT, UEFI, só console |
| `images/utm/OSMarcioLinux-UTM-AArch64-Minimal-v0.2.0.img` | idem (588 MB em disco) — **o arquivo usado no UTM** |

Nenhuma imagem AMD64/VirtualBox foi gerada até hoje (Fase 5 planejada); a
pergunta "ISO x86_64 no UTM ARM64" não se aplica.

## 2. Arquitetura encontrada

- `file`: *DOS/MBR boot sector; partition 1: ID=0xee* → **GPT protetivo**, não ISO 9660 / El Torito.
- `fdisk`: 3 205 120 setores de 512 B; manifesto: ESP FAT32 com
  `EFI/BOOT/BOOTAA64.EFI` (GRUB arm64-efi) + raiz ext4, kernel `6.12.116+deb13-arm64`.
- A imagem estava correta para ARM64/UEFI (já aprovada em 2 boots QEMU/HVF na v0.1.0).

## 3. Causa raiz

A VM **"OSMarcio Linux"** no UTM (lida por AppleScript, sem alterá-la):

```
architecture:aarch64, hypervisor:true, uefi:true, machine:virt
drives: interface:USB,    removable:true,  host size:588   ← a .img como "Boot ISO Image" (CD)
        interface:VirtIO, removable:false, host size:0     ← disco novo VAZIO
displays: virtio-gpu-pci
```

A `.img` (um **disco** GPT) foi anexada pelo assistente do UTM como **mídia
removível/CD USB** e o disco VirtIO criado estava vazio. Arquitetura, firmware
e virtualização estavam certos — o erro foi o **tipo de mídia**.

### Evidência (reprodução no QEMU com a mesma topologia)

`qemu-system-aarch64 -machine virt -accel hvf` + edk2 aarch64 + IMG v0.2.0 como
`usb-storage` `media=cdrom` + qcow2 vazio em `virtio-blk`:

```
Image type X64 can't be loaded on AARCH64 UEFI system.        ← ROM PXE x86 da placa de rede (inofensivo)
BdsDxe: failed to load Boot0001 "UEFI QEMU QEMU USB HARDDRIVE 1-0000:00:02.0-1" ...: Not Found
BdsDxe: failed to load Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0): Not Found
BdsDxe: starting Boot0003 "EFI Internal Shell"
UEFI Interactive Shell v2.2 ... Shell>
```

**Primeira etapa que falha: o firmware UEFI** — ele não encontra partição EFI
na mídia (a GPT de setores de 512 B lida como CD de 2048 B não é reconhecida;
não há catálogo El Torito) e o disco VirtIO está vazio. Bootloader, kernel,
initramfs, rootfs e interface gráfica nunca chegaram a ser executados.

Um segundo problema, independente do boot: **a imagem não tinha desktop
algum** (perfil `desktop` vazio; README/PROGRESS: "Nenhum desktop gráfico
ainda"). Mesmo com o disco correto, só haveria login em texto.

## 4. Correções

### 4.1 Boot no UTM (mídia correta, automatizado)
- `make run-utm` → `scripts/run/utm.sh`: converte a IMG para qcow2 de 20 GiB em
  `out/utm/` (a IMG nunca é alterada), cria a VM por AppleScript com a imagem
  como **disco VirtIO não removível** (QEMU, aarch64, HVF, UEFI, 4 GiB, 4 CPUs,
  `virtio-net-pci` compartilhada, `virtio-gpu-pci`) e a inicia. Se a VM já
  existir, só a inicia (discos nunca recriados/sobrescritos).
- Guia: `docs/installation/UTM_Quick_Start.md` (caminho manual com o aviso
  explícito de **não** usar "Boot ISO Image").

### 4.2 Perfil desktop (necessário para "visualizar a interface gráfica")
- `distro/profiles/desktop/packages.list`: Xorg (driver `modesetting`), LightDM
  + GTK greeter, XFCE (componentes, sem metapacote), tema Greybird-dark +
  ícones elementary-xfce-dark, spice-vdagent, qemu-guest-agent, dependências do
  Terminal (GTK4/VTE) e do runtime CEF. Escolha = **plano B da ADR 0003**
  (XFCE/X11): no UTM com virtio-gpu sem 3D é o caminho mais previsível e o CEF
  roda em X11 nativo.
- `distro/overlays/desktop/`: configuração do LightDM, painel XFCE pronto (sem
  o diálogo de 1º uso), lançadores do Terminal e Browser, apps padrão do XFCE
  (`helpers.rc`), fundo na cor OSMarcio, entradas de e-mail/gerenciador de
  arquivos ocultas (ainda não existem), wrapper `/usr/bin/osmarcio-browser`.
- `scripts/build/customize-rootfs.sh` (perfil ≠ minimal): instala os `.deb`
  do Terminal e do `osmarcio-info`, copia o bundle do Browser para
  `/opt/osmarcio/browser`, habilita `graphical.target` + LightDM, e acrescenta
  `pam_echo` ao PAM do LightDM para que a **senha temporária do 1º boot
  apareça na tela de login gráfica** (antes só aparecia no console texto).
- Rede systemd-networkd/resolved passou a ser habilitada em todos os perfis.

### 4.3 Defeito real encontrado no OSMarcio Browser
A página carregava (HTTP 200, título correto, sandbox ativo), mas **a área da
página ficava vazia** — 1 única cor na captura de tela. Causa:
`BoxLayoutSettings::default()` do cef-rs zera a struct, então
`cross_axis_alignment = START` (0) em vez de `STRETCH` (3) do C++ do CEF; num
layout vertical a `BrowserView` ficava com a largura preferida (0 px).
Correção: `vbox_settings()` com `AxisAlignment::STRETCH` em
`apps/osmarcio-browser/src/ui.rs`. Depois: 221 cores na área da página e
example.com visível. O teste antigo (`test-browser-gui.sh`) só olhava logs e
nunca poderia ter pego isso.

### 4.4 Outros bugs corrigidos
- `scripts/build/deb.sh`: aborta silenciosamente (grep + `pipefail`) para apps
  sem `.desktop` → o `.deb` do `osmarcio-info` nunca era gerado.
- `scripts/test/test-boot-qemu-arm64.sh`: passou a dar placa de vídeo à VM
  (sem ela o Xorg não sobe e o teste do desktop não faz sentido).
- `build-disk-image.sh`: nome da imagem desktop agora é o pedido
  (`…-AArch64-Desktop-v<versão>.img`).

## 5. Arquivos modificados

| Novos | Alterados |
|---|---|
| `distro/profiles/desktop/packages.list` | `Makefile` |
| `distro/overlays/desktop/**` | `scripts/build/customize-rootfs.sh` |
| `distro/overlays/common/usr/lib/osmarcio/selftest-desktop.sh` | `scripts/build/deb.sh` |
| `scripts/run/utm.sh` | `scripts/image/build-disk-image.sh` |
| `scripts/test/test-desktop-qemu-arm64.sh` | `scripts/run/qemu-arm64.sh` (`--gpu-headless`, `--monitor`) |
| `scripts/test/qemu-monitor.py` | `scripts/test/test-boot-qemu-arm64.sh` |
| `docs/installation/UTM_Quick_Start.md` | `scripts/setup/setup-macos.sh` (UTM/utmctl/imagem) |
| este relatório | `distro/overlays/common/usr/lib/osmarcio/selftest.sh` (checagens pré-login) |
| | `apps/osmarcio-browser/src/ui.rs` · `README.md` · `docs/utm/UTM_Guide.md` · `PROGRESS.md` · `CHANGELOG.md` |

Não tocados: `/Users/marcio/_learn/OSMarcio`, imagens Minimal v0.1.0/v0.2.0,
`out/vm/` (disco persistente), a VM "OSMarcio Linux" original do UTM, kernel,
alvos VirtualBox/ROCK Pi. Builds intermediários da imagem desktop desta
sessão foram movidos (não apagados) para `out/superseded/*.build1|build2`.

## 6. Comandos executados (principais)

```
osascript -e 'tell application "UTM" to get configuration of virtual machine named "OSMarcio Linux"'
qemu-system-aarch64 … usb-storage media=cdrom (reprodução da falha)
scripts/build/in-builder.sh … deb.sh osmarcio-terminal | deb.sh osmarcio-info | browser-bundle.sh
scripts/image/build-disk-image.sh arm64 desktop utm      (= make image-utm)
make test-desktop   make test-boot   make verify-images   make doctor-mac   make help
make run-utm        utmctl exec / file pull  (testes dentro do app UTM)
```

## 7. Resultado dos builds

| Artefato | Resultado |
|---|---|
| `out/debs/osmarcio-terminal_0.2.0_arm64.deb`, `osmarcio-info_0.2.0_arm64.deb` | OK |
| `out/browser-bundle-arm64-v0.2.0/` (332 MB; `osmarcio-browser` e `libcef.so` ELF aarch64) | OK |
| **`images/utm/OSMarcioLinux-UTM-AArch64-Desktop-v0.2.0.img`** | OK — 2825 MiB lógicos, 1,5 GB em disco, 434 pacotes, SHA-256 `8780d9b0…3e62` (`make verify-images` OK) |
| `cargo clippy -p osmarcio-browser -D warnings` | **2 avisos pré-existentes** (`collapsible_if`, ui.rs) — não corrigidos (fora do escopo) |

## 8. Boot no QEMU (QEMU 11 + HVF, mesma máquina `virt`/edk2/virtio do UTM)

`make test-boot` → `reports/tests/boot-arm64-20261113T113805Z.md`: **0 falhas**,
2 boots, persistência PASS. Sequência verificada: firmware → GRUB → kernel →
initramfs → raiz ext4 → systemd → rede/DNS/HTTPS/APT → `graphical.target` →
LightDM + Xorg + greeter.

`make test-desktop` → `reports/tests/desktop-arm64-20261113T114410Z.md`: **0 falhas**.
Login digitado no greeter pelo monitor do QEMU (`sendkey`), com a troca
obrigatória de senha; então: sessão XFCE (x11), xfwm4, painel, xfdesktop,
1280x800, Terminal, Browser, renderização da página. Capturas em
`reports/tests/screenshots/desktop-arm64-20261113T114410Z-*.png`.

Execuções anteriores mantidas como histórico: `…112250Z` (antes da correção do
browser — passou nos logs, mas a página estava em branco), `…112503Z`
(inspeção), `…113333Z` (1 falha: critério de pixel errado, supunha fundo claro), `…113541Z` (0 falhas; `make` saía com 1 por bug do trap no bash 3.2, corrigido).

## 9. Boot no **app UTM** (teste real, não só QEMU)

`make run-utm` criou e iniciou **"OSMarcio Linux 0.2.0 Desktop"** (config lida
de volta: aarch64, hypervisor, UEFI, VirtIO, virtio-gpu-pci, shared). Evidências
coletadas pelo `qemu-guest-agent` (`utmctl ip-address/exec/file pull`):

- IP `192.168.64.19`; `systemctl is-system-running` = **running**; raiz expandida a 20 GB.
- LightDM ativo; Xorg em `vt7`; `modeset(0)` em `Virtual-1` 1280x800; HTTPS 200.
- Captura do display **de dentro da VM** (o macOS não me dá permissão de
  gravação de tela): greeter OSMarcio com a senha temporária —
  `reports/tests/screenshots/utm-app-20261113T1145Z-greeter.png`.
- Login real no greeter (entrada sintética XTest via `xdotool`, não o teclado
  do Mac) + troca de senha + o mesmo autoteste de sessão:
  `reports/tests/utm-app-20261113T1145Z-osmtest.log` — **12 PASS, 0 FAIL**;
  capturas `utm-app-20261113T1145Z-browser.png` / `-desktop.png`.
- Depois a VM foi **devolvida ao estado de 1º boot** (sessão encerrada, nova
  senha temporária pelo próprio `firstboot.sh`, ImageMagick de teste removido).
  Ela está ligada no UTM, na tela de login.

**Não verificado por mim:** a janela do UTM vista pelos seus olhos, teclado e
mouse físicos do Mac dentro dela e o redimensionamento dinâmico.

## 10. Status

| Item | Status | Evidência |
|---|---|---|
| Boot UEFI ARM64 | **Funciona** (QEMU e UTM) | seções 8–9 |
| Display manager (LightDM) | **Funciona** | greeter capturado no QEMU e no UTM |
| Sessão gráfica / WM / painel / menu | **Funciona** (XFCE 4.20, X11, xfwm4) | OSMTEST + capturas |
| Resolução | 1280x800 fixa nos testes; dinâmica depende do spice-vdagent no UTM (**não testada**) | xrandr |
| Teclado / mouse | Teclado USB + tablet USB funcionam no QEMU; no UTM só entrada sintética | — |
| Ícones / tema OSMarcio | Greybird-dark + elementary-xfce-dark + cor OSMarcio. **Não há tema próprio** | capturas |
| **OSMarcio Terminal** | **Funciona**: abre, aceita digitação, bash executa (`OSMTERM-42 osmarcio /bin/bash`) | OSMTEST |
| **OSMarcio Browser** | **Funciona** em ARM64: binários aarch64, nenhuma lib ausente, CEF 156 inicializa, janela abre, HTTPS 200 (example.com), página **visível**, sandbox (seccomp=2 + PID namespace), nenhum `--no-sandbox`, ~740 MiB RSS | OSMTEST + pngstat |
| Tela "Sobre" | **Não implementada** como janela; `osmarcio-info` (CLI Rust) funciona no terminal | captura do terminal |

## 11. Limitações

- Fontes CJK ausentes (caracteres chineses/japoneses aparecem como □).
- Sem aceleração 3D (glamor falha → renderização por software); suficiente
  para o desktop e o browser nos testes.
- `osmarcio-info` mostra "Desktop: —" quando iniciado fora do ambiente da
  sessão (nos testes o terminal herda o ambiente do autoteste, não do XFCE).
- Gerenciador de arquivos, editor, e-mail, configurações próprias: inexistentes (fases futuras).
- Clipboard/resolução dinâmica no UTM não testados; a imagem tem `spice-vdagent`.
- Só ARM64. VirtualBox/x86_64 e ROCK Pi 4A continuam como estavam (planejado/bloqueado).

## 12. Próximas ações

1. Você: abrir a VM "OSMarcio Linux 0.2.0 Desktop" no UTM e fazer o 1º login
   (senha temporária na tela) — e remover a VM antiga "OSMarcio Linux" quando quiser.
2. Corrigir os 2 avisos de clippy do browser e incluir a checagem de pixels no `test-browser-gui.sh`.
3. Pacote `fonts-noto-cjk` (opcional, ~60 MB) e ajuste de `osmarcio-info --desktop` dentro da sessão.
4. Janela "Sobre o OSMarcio Linux" gráfica (Fase 13) usando `osmarcio_core::sysinfo`.
