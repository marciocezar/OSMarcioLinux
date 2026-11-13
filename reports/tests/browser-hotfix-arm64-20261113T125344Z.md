# Teste do hotfix do OSMarcio Browser — OSMarcioLinux-UTM-AArch64-Desktop-v0.2.1.img

- Data (UTC): 20261113T125344Z · Host: Darwin arm64, QEMU emulator version 11.0.1
- Sessão gráfica: login real no LightDM (ver `reports/tests/desktop-arm64-20261113T125344Z.md`)
- SHA-256 da imagem: `eaeaf482e2906d88ea845d9a3e6d1b1706c42b86c354b3a697dc8a6799f8071d`
- Resultado: **32 PASS, 0 FAIL**

```
PASS fechar-x-1-aba janela fechada, processo encerrado normalmente em 0s, 0 subprocessos
PASS tres-abas abertas
PASS fechar-altf4-3-abas janela fechada, processo encerrado normalmente em 0s, 0 subprocessos
PASS carregando página em carregamento (servidor lento)
PASS fechar-ctrlq-carregando janela fechada, processo encerrado normalmente em 1s, 0 subprocessos
PASS download iniciado (download_start path=/home/osmarcio/Downloads/grande.bin)
PASS fechar-x-download-ativo janela fechada, processo encerrado normalmente em 0s, 0 subprocessos
PASS menu aberto pelo botão ☰
PASS fechar-menu janela fechada, processo encerrado normalmente em 0s, 0 subprocessos
PASS ctrl-w fechou só a aba (janela continua aberta)
PASS fechar-ctrlw-ultima-aba janela fechada, processo encerrado normalmente em 1s, 0 subprocessos
RESULTADO(fechar): 11 passaram, 0 falharam
PASS reabrir navegador abriu e carregou de novo
LIMITACAO wm-class ausente (WM_CLASS:  not found.)
PASS icone-janela _NET_WM_ICON: Icon (16 x 16) Icon (128 x 128) 
PASS sobre-abrir browser=0.2.1 chromium=156.0.8078.88 cef=156.0.26 cef_compilado=156.0.26+g8ea0c73+chromium-156.0.8078.88 cef_rs=156.4.0+156.0.26 arch=aarch64 sistema=OSMarcio Linux 0.2.1 runtime_confere=true
PASS sobre-janela X11 27262987
PASS sobre-chromium igual ao --version (156.0.8078.88)
PASS sobre-runtime libcef.so carregado == versão compilada
RESULTADO(sobre-abrir): 6 passaram, 0 falharam
PASS sobre-verificar Fonte de atualização não configurada (nem repositório APT do OSMarcio Browser nem manifest_url em /etc/osmarcio/browser-updates.toml
PASS sobre-verificar estado 'fonte não configurada' (padrão do pacote)
PASS sobre-copiar texto copiado contém as versões (colado na barra de endereços)
RESULTADO(sobre-verificar): 3 passaram, 0 falharam
PASS sobre-fechar botão Fechar fechou só o Sobre; navegador segue aberto
PASS fechar-x-com-sobre-aberto janela fechada, processo encerrado normalmente em 0s, 0 subprocessos
RESULTADO(sobre-fechar): 2 passaram, 0 falharam
PASS atualizacao-fonte-ausente: Fonte de atualização não configurada (nem repositório APT do OSMarcio Browser nem manifest_url em /etc/osmarcio/browser-updates.toml).
PASS atualizacao-ja-atualizado: O OSMarcio Browser está atualizado (0.2.1).
PASS atualizacao-nova-homologada: Versão homologada 9.9.9 (Chromium 157.0.0.0) publicada, mas não há repositório APT assinado configurado para instalá-la com segurança.
PASS atualizacao-nao-homologada: Existe a versão 9.9.9, ainda não homologada para o OSMarcio Linux — não oferecida.
PASS atualizacao-incompativel: A versão 9.9.9 não tem pacote para esta arquitetura (arm64).
PASS atualizacao-http-recusado: Não foi possível verificar atualizações: manifest_url recusada (use https:// ou file:///): http://deb.debian.org/m.toml
PASS atualizacao-dns-falha: Não foi possível verificar atualizações: sem conexão (nome do servidor não resolvido)
PASS atualizacao-tls-falha: Não foi possível verificar atualizações: falha TLS/certificado — a fonte não foi aceita
PASS atualizacao-resposta-invalida: Não foi possível verificar atualizações: manifesto inválido: TOML parse error at line 1, column 11
PASS atualizacao-sem-internet: Não foi possível verificar atualizações: sem conexão com o servidor de atualizações
RESULTADO(atualizacoes): 10 passaram, 0 falharam
```

## Capturas de tela
![01-sobre-e-barra-de-tarefas](screenshots/browser-hotfix-arm64-20261113T125344Z-01-sobre-e-barra-de-tarefas.png)
![02-alt-tab](screenshots/browser-hotfix-arm64-20261113T125344Z-02-alt-tab.png)
![03-menu-internet](screenshots/browser-hotfix-arm64-20261113T125344Z-03-menu-internet.png)
![04-sobre-verificar-atualizacoes](screenshots/browser-hotfix-arm64-20261113T125344Z-04-sobre-verificar-atualizacoes.png)
