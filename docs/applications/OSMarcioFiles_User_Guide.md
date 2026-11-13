# OSMarcio Files — guia do usuário

Abra pelo menu **OSMarcio → Sistema → OSMarcio Files**, pelo ícone de pasta do
painel, ou no terminal com `osmarcio-files [pasta]`. Qualquer pasta aberta por
outro programa também abre no Files (aplicativo padrão para pastas).

## Navegar
| Ação | Como |
|---|---|
| Entrar numa pasta | duplo clique ou Enter |
| Voltar / avançar / pasta acima | botões ◀ ▶ ▲, Alt+← / Alt+→ / Alt+↑ ou Backspace |
| Pasta pessoal | botão ⌂ ou Alt+Home |
| Digitar um caminho | Ctrl+L (aceita `~/…`; `lixeira:` abre a lixeira) |
| Ir para uma parte do caminho | clique na trilha abaixo da barra |
| Atualizar | F5 |
| Mostrar/ocultar arquivos ocultos | Ctrl+H |
| Lista detalhada / ícones | Ctrl+2 / Ctrl+1 (ou menu ☰) |
| Ordenar por nome, tipo, tamanho ou data | menu ☰ → Ordenar por |
| Buscar | Ctrl+F (marque "Incluir subpastas" para buscar recursivamente; Esc fecha) |

A barra lateral mostra a pasta pessoal, as pastas do usuário que existem
(Documentos, Downloads…), Lixeira, favoritos, Sistema de arquivos e volumes
montados.

## Operações
| Ação | Como |
|---|---|
| Nova pasta | Ctrl+Shift+N ou clique direito → Nova pasta |
| Novo arquivo vazio | clique direito → Novo arquivo vazio |
| Selecionar vários | Ctrl+clique, Shift+clique, Ctrl+A |
| Copiar / recortar / colar | Ctrl+C / Ctrl+X / Ctrl+V |
| Renomear | F2 |
| Mover para a lixeira | Delete |
| Restaurar da lixeira | na Lixeira, Ctrl+Shift+R ou clique direito → Restaurar |
| Excluir permanentemente | Shift+Delete (pede confirmação; não tem volta) |
| Esvaziar lixeira | menu ☰ → Esvaziar lixeira… |
| Propriedades | Alt+Enter ou clique direito → Propriedades |
| Cancelar cópia/movimentação | Esc ou botão Cancelar na barra de progresso |

Se já existir um item com o mesmo nome no destino, a cópia recebe outro nome
("foto (2).jpg") — nada é sobrescrito. Arquivos abrem no aplicativo padrão do
tipo (texto → Mousepad, imagem → Ristretto). Programas e scripts **não são
executados** pelo Files, por segurança: use o terminal.

## Mensagens comuns
"permissão negada" — sua conta não tem acesso (ex.: `/root`). "Nenhum aplicativo
abre…" — não há programa instalado para o tipo (ex.: PDF). "sem espaço no destino".
