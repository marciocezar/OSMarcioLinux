# OSMarcio Browser — Solução de problemas

| Sintoma | Diagnóstico | Solução |
|---|---|---|
| X não fecha a janela (versões ≤ 0.2.0) | defeito de ciclo de vida corrigido em 0.2.1 ([detalhes](OSMarcioBrowser_Window_Lifecycle.md)) | atualize para 0.2.1; até lá: Ctrl+W em cada aba |
| Sobram processos depois de fechar | `pgrep -u $USER -fa osmarcio-browser` | não deve ocorrer em 0.2.1; reporte com o log abaixo |
| Página em branco, interface ok | versões antigas sem a correção de layout; ou falta de rede | `RUST_LOG=info osmarcio-browser https://example.com` e procure `load_end` |
| Ícone do Chromium em vez do OSMarcio | pacote antigo ou cache de ícones | `dpkg -l osmarcio-browser`; `sudo gtk-update-icon-cache -f /usr/share/icons/hicolor` |
| "Fonte de atualização não configurada" | padrão: não há repositório APT assinado ainda | normal; ver [Atualizações](OSMarcioBrowser_Updates.md) |
| "falha TLS/certificado" na verificação | a fonte tem certificado inválido — recusada de propósito | corrija a fonte; nunca desative a verificação |
| `dpkg` pergunta sobre `browser-updates.toml` | você editou o conffile | escolha manter (N) ou o do mantenedor (Y) |
| "não execute o navegador como root" | sandbox do Chromium exige usuário comum | rode como usuário normal |

## Logs
```bash
RUST_LOG=info osmarcio-browser https://example.com      # eventos "osmb:" no terminal
cat ~/.cache/osmarcio/browser/cef.log                     # log do Chromium (avisos e erros)
osmarcio-browser --version                                # versões reais
osmarcio-browser --check-updates                          # estado das atualizações
journalctl --user -b | grep -i osmarcio
```

Linhas úteis do log ao fechar: `fechar janela solicitado abas=N pode_fechar=…`,
`aba fechando`, `aba fechada`, `janela destruída`, `encerrando loop de mensagens`,
`encerrado`.
