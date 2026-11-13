<#
.SYNOPSIS
  Cria e inicia uma VM de TESTE do OSMarcio Linux no VirtualBox (x86_64, UEFI, NAT).
.DESCRIPTION
  1. detecta o VirtualBox e localiza VBoxManage;  2. verifica a ISO (e o .sha256);
  3. cria a VM (recusa se já existir — nunca sobrescreve);  4. Debian 64-bit;
  5. firmware EFI;  6. RAM/CPU;  7. cria VDI novo;  8. rede NAT;
  9. anexa a ISO;  10. inicia a VM.
.NOTES
  Status: escrito sem acesso a Windows/VirtualBox e sem ISO x86_64 ainda
  (Fase 5). NÃO TESTADO. Veja PROGRESS.md.
.EXAMPLE
  .\scripts\windows\create-virtualbox-vm.ps1 -IsoPath .\images\virtualbox\OSMarcioLinux-x86_64-Desktop-v0.1.0.iso
#>
param(
  [Parameter(Mandatory = $true)][string]$IsoPath,
  [string]$VmName = 'OSMarcioLinux-Teste',
  [int]$MemoryMB = 4096,
  [int]$Cpus = 2,
  [int]$DiskGB = 32,
  [switch]$NoStart
)
$ErrorActionPreference = 'Stop'

function Fail($msg) { Write-Host "[erro] $msg" -ForegroundColor Red; exit 1 }
function Info($msg) { Write-Host "[osmarcio] $msg" -ForegroundColor Cyan }

# 1-2. VirtualBox / VBoxManage
$vbox = $null
foreach ($p in @("$env:VBOX_MSI_INSTALL_PATH", "$env:ProgramFiles\Oracle\VirtualBox")) {
  if ($p -and (Test-Path (Join-Path $p 'VBoxManage.exe'))) { $vbox = Join-Path $p 'VBoxManage.exe'; break }
}
if (-not $vbox) { Fail 'VBoxManage.exe não encontrado. Instale o VirtualBox (winget install Oracle.VirtualBox).' }
Info "VirtualBox $(& $vbox --version)"

# 3. ISO
$iso = Resolve-Path $IsoPath -ErrorAction SilentlyContinue
if (-not $iso) { Fail "ISO não encontrada: $IsoPath" }
$shaFile = "$iso.sha256"
if (Test-Path $shaFile) {
  $expected = ((Get-Content $shaFile -Raw).Split(' ')[0]).Trim().ToLower()
  $actual = (Get-FileHash $iso -Algorithm SHA256).Hash.ToLower()
  if ($expected -ne $actual) { Fail "SHA-256 da ISO não confere ($actual)" }
  Info 'SHA-256 da ISO conferido'
} else { Write-Host "[aviso] sem $shaFile; checksum não verificado" -ForegroundColor Yellow }

# 4. Não sobrescrever VMs existentes
$existing = & $vbox list vms
if ($existing -match "`"$([regex]::Escape($VmName))`"") { Fail "a VM '$VmName' já existe; escolha outro -VmName (nada foi alterado)." }

$base = Join-Path $env:USERPROFILE 'VirtualBox VMs'
$vdi = Join-Path $base "$VmName\$VmName.vdi"
if (Test-Path $vdi) { Fail "o disco $vdi já existe; não será sobrescrito." }

# 5-8. Criação
Info "criando VM '$VmName'"
& $vbox createvm --name $VmName --ostype Debian_64 --register --basefolder $base | Out-Null
& $vbox modifyvm $VmName --firmware efi --memory $MemoryMB --cpus $Cpus `
    --graphicscontroller vmsvga --vram 128 --nic1 nat --mouse usbtablet `
    --rtc-use-utc on --boot1 dvd --boot2 disk --boot3 none --boot4 none
& $vbox createmedium disk --filename $vdi --size ($DiskGB * 1024) --format VDI | Out-Null
& $vbox storagectl $VmName --name SATA --add sata --controller IntelAhci --portcount 2
& $vbox storageattach $VmName --storagectl SATA --port 0 --device 0 --type hdd --medium $vdi
# 9. ISO
& $vbox storageattach $VmName --storagectl SATA --port 1 --device 0 --type dvddrive --medium $iso

Info "VM criada: UEFI, $MemoryMB MiB, $Cpus vCPU, VDI $DiskGB GiB, rede NAT"
# 10. Início
if (-not $NoStart) { & $vbox startvm $VmName --type gui | Out-Null; Info 'VM iniciada' }
