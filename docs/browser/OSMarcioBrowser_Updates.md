# OSMarcio Browser — Política de atualizações

## Princípio
O Chromium/CEF **não** é um componente substituível à parte: o binário Rust é
compilado contra uma versão exata do CEF (API C com *hash* verificado na
inicialização). Atualizar o Chromium exige nova versão do cef-rs, recompilação,
testes de sandbox e um **novo pacote `osmarcio-browser`**. Por isso:

- o navegador **nunca** baixa, substitui ou executa binários;
- o processo do navegador **nunca** roda como root nem instala nada;
- instalação só por **repositório APT assinado** (`apt` verifica as assinaturas
  no `apt update`), feita pelo usuário com `sudo` (autorização administrativa).

## Infraestrutura existente (auditoria)
| Item | Existe? |
|---|---|
| Pacotes `.deb` | **Sim** (desde 0.2.1: `make browser-package`) |
| Release local com SHA-256 | **Sim** (`make browser-release` → `images/releases/`) |
| Repositório APT próprio assinado | **Não** |
| Servidor/manifesto público de versões | **Não** |
| Assinatura de pacotes | **Não** (só checksum) |

Conclusão: **não há canal assinado**, então o botão **verifica e informa**, mas
não instala. A arquitetura está pronta para o canal APT quando ele existir.

## Como a verificação decide (`osmarcio_core::updates`, testado)
1. Pacote instalado (`dpkg-query`) mais novo que o processo → **Reinício necessário**.
2. `apt-cache policy osmarcio-browser` com candidato mais novo → **Atualização disponível**
   ("sudo apt install --only-upgrade osmarcio-browser").
3. `manifest_url` em `/etc/osmarcio/browser-updates.toml` (conffile, vazio por padrão):
   manifesto TOML de releases, lido com `curl --proto =https,file` (TLS verificado,
   máx. 1 MiB). Uma release só é oferecida se for **mais nova, da mesma
   arquitetura, `homologated = true` e com `sha256`**.
4. Nada configurado → **Fonte de atualização não configurada**.

```toml
[[release]]
version = "0.2.2"
arch = "arm64"
chromium = "156.0.8078.88"
cef = "156.0.26"
homologated = true
sha256 = "<64 hex do .deb>"
```

## Estados exibidos
| Estado | Quando |
|---|---|
| Verificando atualizações… | durante a consulta (thread separada; a janela não trava) |
| Browser atualizado | nada mais novo nas fontes configuradas |
| Atualização disponível (APT) | candidato APT assinado mais novo |
| Versão homologada publicada, sem canal APT | manifesto lista versão homologada, mas não há como instalar com segurança |
| Versão ainda não homologada | mais nova, mas `homologated = false` ou sem `sha256` |
| Sem pacote para esta arquitetura | mais nova só para outra arch |
| Reinício necessário | pacote instalado > processo em execução |
| Não foi possível verificar: sem conexão / falha TLS / protocolo recusado / manifesto inválido | erros de rede/fonte |
| Fonte de atualização não configurada | padrão atual do pacote |
| *Atualização em andamento* | **não se aplica**: a instalação é feita pelo `apt`, fora do navegador |

Uma versão nova do Chromium upstream **não** gera "atualização disponível":
só uma release homologada do pacote OSMarcio Browser.

## Testes
- Unitários (`cargo test -p osmarcio-core`): versões, homologação, checksum, arquitetura, URL, `apt-cache policy`.
- Na VM (`test-browser-hotfix.sh atualizacoes`, CLI `--check-updates` — mesma função do botão):
  fonte ausente, já atualizado, nova homologada, não homologada, incompatível,
  `http://` recusado, DNS inexistente, certificado expirado (badssl.com), resposta
  não-TOML, **sem internet** (rota padrão removida).
- Reinício necessário: verificado com o pacote 0.2.1 instalado e um processo 0.2.0 em execução.
- **Bloqueado:** instalação com autorização e verificação pós-atualização — exigem
  um repositório APT assinado, que não existe. Não foi simulado.
