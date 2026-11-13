# Revisão de segurança — v0.1.0 (Minimal ARM64)

## Atendido (verificado no autoteste de boot)

| Requisito | Como | Verificação |
|---|---|---|
| Sem senha fixa / credenciais na imagem | usuário criado no 1º boot com senha aleatória (12 caracteres) e troca obrigatória (`chage -d 0`) | `user-pw-temp`, `no-other-pw` |
| Root sem login | `passwd -l root` | `root-locked` |
| Usuário comum + sudo | `osmarcio` no grupo `sudo` | `user-exists` |
| Identidade única por máquina | `machine-id` vazio na imagem; sem chaves SSH | build |
| Pacotes assinados | `debian-archive-keyring`; `AllowInsecureRepositories` não habilitado | `apt-signed` |
| Atualizações de segurança | `trixie-security` configurado | `sources` no build |
| TLS validado | `ca-certificates`; `curl` retorna `ssl_verify_result=0` | `https` |
| Build não toca discos do host | sem loop devices/mount; `refuse_overwrite` | revisão de código |

## Riscos e pendências

| # | Item | Severidade | Plano |
|---|---|---|---|
| 1 | Senha temporária visível na tela de login até a troca (`/etc/issue.d`) | Média para VMs compartilhadas | aceitável para VM pessoal; no instalador (Fase 15) a senha será escolhida pelo usuário |
| 2 | `nftables` instalado sem regras ativas | Média | política padrão "entrada negada exceto established/ICMP" na Fase 6 |
| 3 | AppArmor não habilitado/ajustado | Baixa | avaliar com o perfil Desktop |
| 4 | Imagens não assinadas (só SHA-256) | Média para distribuição pública | assinatura GPG de artefatos na Fase 18 |
| 5 | Container de build `--privileged` | Baixa (escopo: VM do Docker) | documentado na ADR 0006 |
| 6 | Toolchain/crates baixados da internet sem lockfile de hashes próprio | Baixa | `Cargo.lock` versionado; `cargo vet`/`cargo deny` futuros |
| 7 | Sandbox do Chromium | — | requisito da POC (ADR 0005); `--no-sandbox` proibido |
