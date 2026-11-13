# OSMarcio Files — implementação (0.3.0)

**Estado: Implementado e testado (ARM64, QEMU/HVF)** — ver resultado final em
[OSMarcioLinux_Desktop_Regression_Report.md](OSMarcioLinux_Desktop_Regression_Report.md).

## Entregue
`apps/osmarcio-files/` (Rust + GTK4 + GIO), pacote `osmarcio-files_0.3.0_arm64.deb`,
ícone original, `.desktop` (Sistema; `MimeType=inode/directory`), aplicativo padrão
para pastas (`/etc/xdg/mimeapps.list`), helper FileManager do XFCE, lançador no painel.
Arquitetura: [docs/applications/OSMarcioFiles_Architecture.md](../docs/applications/OSMarcioFiles_Architecture.md) ·
guia: [OSMarcioFiles_User_Guide.md](../docs/applications/OSMarcioFiles_User_Guide.md).

| Área | Implementado |
|---|---|
| Navegação | abrir, voltar, avançar, acima, home, atualizar, caminho manual (`~`, `lixeira:`), trilha clicável, duplo clique, teclado |
| Barra lateral | pasta pessoal, pastas XDG existentes (Área de trabalho, Documentos, Downloads, Imagens, Músicas, Vídeos), Lixeira, favoritos GTK, Sistema de arquivos, volumes montados |
| Visões | lista detalhada (nome, tamanho, tipo, modificação, permissões) e ícones; ordenação por nome/tipo/tamanho/data; ocultos Ctrl+H |
| Operações | nova pasta, arquivo vazio, copiar/mover múltiplos (arquivos e pastas), renomear, lixeira, restaurar, esvaziar, exclusão permanente com confirmação, propriedades, busca (atual/recursiva, cancelável), Ctrl+A/C/X/V |
| Operações longas | thread + progresso em bytes + cancelamento; sem parcial; sucesso só ao final |
| Segurança | não roda como root; sem sobrescrita; recursão não segue links; diretórios críticos protegidos; scripts/programas não executados; erros de permissão/espaço/volume/I-O legíveis |

## Testes
| Teste | Resultado |
|---|---|
| `make files-test` — operações reais em /tmp | 15/15 (cópia de árvore com link, nomes únicos, proteção, cancelamento sem parcial, mover/renomear/criar, busca sem seguir links, exclusão sem seguir links, lixeira: listar/restaurar/esvaziar/traversal, listagem Unicode/links) |
| VM — fases files-* (20 verificações) | criar, copiar, mover, renomear, lixeira, restaurar, cancelar 2 GiB, copiar 2 GiB, /root negado, propriedades, busca, ocultos, abrir texto (Mousepad via MIME), script não executado, exclusão permanente, fechar |

## Defeitos encontrados pelos testes gráficos e corrigidos
1. `total_size` contava o tamanho do próprio link → progresso nunca chegava a 100%.
2. Renomear selecionava o nome inteiro (o foco do GtkEntry desfazia a seleção do radical) → a extensão se perdia; depois, a seleção em *idle* podia engolir a 1ª letra digitada → seleção síncrona após o foco.
3. Após navegar pela barra de endereço o foco ficava nela → Ctrl+C/V/Delete agiam no texto → foco volta à lista.
4. Cancelar uma cópia abria diálogo modal de "erro" → agora só a barra de status.
5. Na lixeira a trilha (rótulo) era mais baixa que nas pastas (botões) → lista deslocada; coluna Nome espremida por caminhos longos → mesma altura e colunas secundárias com reticências.

## Limitações
Sem gvfs/udisks (sem Recentes, montagem de dispositivos ou protocolos remotos);
conflitos sempre resolvidos por renomeação automática; área de transferência de
arquivos interna ao app; busca limitada a 5000 resultados; descrições MIME em
inglês quando o shared-mime-info não traz tradução.
