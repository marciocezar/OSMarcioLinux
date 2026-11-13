<#
.SYNOPSIS
  OSMarcio Linux - diagnóstico/preparação do Windows (host do VirtualBox).
.DESCRIPTION
  -Check  apenas diagnostica.
  Sem -Check: diagnostica e mostra os comandos winget para o que faltar.
  Este script NÃO instala nada sozinho e NÃO exige Homebrew.
  O build das imagens acontece no WSL2 (scripts/setup/setup-wsl.sh), na MESMA
  pasta C:\Users\<usuário>\_code\OSMarcioLinux.
.NOTES
  Status: escrito sem acesso a uma máquina Windows; ainda NÃO testado.
#>
param([switch]$Check)
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$Version = (Get-Content (Join-Path $Root 'VERSION') -Raw).Trim()
$missing = @()

function Row($state, $name, $detail) {
  $color = @{ OK = 'Green'; FALTA = 'Red'; AVISO = 'Yellow' }[$state]
  Write-Host ("  {0,-7} {1,-22} {2}" -f $state, $name, $detail) -ForegroundColor $color
}

Write-Host "OSMarcio Linux $Version - diagnóstico do Windows"
Row OK 'PowerShell' $PSVersionTable.PSVersion.ToString()
Row OK 'Windows' ([System.Environment]::OSVersion.VersionString)
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq 'AMD64') { Row OK 'Arquitetura' $arch } else { Row AVISO 'Arquitetura' "$arch (o alvo VirtualBox é x86_64)" }

# WSL2
try {
  $wsl = (wsl.exe --status 2>&1) -join ' '
  if ($LASTEXITCODE -eq 0) { Row OK 'WSL' 'instalado' } else { throw 'x' }
  $distros = (wsl.exe -l -v 2>&1) -join ' '
  if ($distros -match '\s2\s') { Row OK 'WSL2 distro' 'versão 2 encontrada' } else { Row AVISO 'WSL2 distro' 'nenhuma distro na versão 2' }
} catch { Row FALTA 'WSL' 'wsl --install -d Debian'; $missing += 'wsl --install -d Debian' }

# VirtualBox
$vbox = $null
foreach ($p in @("$env:VBOX_MSI_INSTALL_PATH", "$env:ProgramFiles\Oracle\VirtualBox")) {
  if ($p -and (Test-Path (Join-Path $p 'VBoxManage.exe'))) { $vbox = Join-Path $p 'VBoxManage.exe'; break }
}
if ($vbox) { Row OK 'VirtualBox' (& $vbox --version) } else { Row FALTA 'VirtualBox' 'winget install Oracle.VirtualBox'; $missing += 'winget install Oracle.VirtualBox' }

# Git
if (Get-Command git -ErrorAction SilentlyContinue) { Row OK 'Git' (git --version) } else { Row FALTA 'Git' 'winget install Git.Git'; $missing += 'winget install Git.Git' }

# Pasta do projeto
$expected = Join-Path $env:USERPROFILE '_code\OSMarcioLinux'
if ($Root -ieq $expected) { Row OK 'Pasta do projeto' $Root } else { Row AVISO 'Pasta do projeto' "$Root (recomendado: $expected)" }

Write-Host ''
if ($missing.Count -eq 0) { Write-Host 'Tudo pronto.'; exit 0 }
if ($Check) { exit 1 }
Write-Host 'Comandos sugeridos (execute você mesmo, revisando cada um):'
$missing | ForEach-Object { Write-Host "  $_" }
exit 1
