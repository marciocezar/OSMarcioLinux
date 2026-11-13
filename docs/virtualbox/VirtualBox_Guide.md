# Guia — OSMarcio Linux no VirtualBox (Windows x86_64)

> **Situação:** ainda não existe ISO x86_64 (Fase 5). Este guia descreve o
> fluxo planejado e o script já escrito, que **não foi testado**.

## Fluxo planejado

1. No WSL2, na pasta compartilhada `/mnt/c/Users/<você>/_code/OSMarcioLinux`:
   `make doctor-wsl`, depois `make iso-x86_64` (Fase 5).
2. No PowerShell, na mesma pasta `C:\Users\<você>\_code\OSMarcioLinux`:

```powershell
.\scripts\setup\setup-windows.ps1 -Check
```

```powershell
.\scripts\windows\create-virtualbox-vm.ps1 -IsoPath .\images\virtualbox\OSMarcioLinux-x86_64-Desktop-v0.1.0.iso
```

O script: acha `VBoxManage`, confere o SHA-256 da ISO, **recusa** criar se
a VM ou o VDI já existirem, cria VM Debian 64-bit com **EFI**, 4 GiB de
RAM, 2 vCPUs, VDI de 32 GiB, rede **NAT**, VMSVGA, anexa a ISO e inicia.

## Teste completo exigido antes de declarar suporte

Boot pela ISO → instalador → instalação no VDI → reboot → boot pelo VDI →
login → desktop → terminal → rede → browser → APT → persistência.
