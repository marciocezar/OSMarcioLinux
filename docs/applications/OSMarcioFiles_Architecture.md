# OSMarcio Files — arquitetura

Gerenciador de arquivos em Rust (GTK4 + GIO), executável `osmarcio-files`,
pacote `osmarcio-files`, app id `org.osmarcio.Files`. Diagrama: página 10.

## Módulos (`apps/osmarcio-files/src/`)
| Arquivo | Responsabilidade | Testes |
|---|---|---|
| `main.rs` | recusa root; abre a home, a pasta da linha de comando ou a recebida por MIME (`HANDLES_OPEN`) | — |
| `ui.rs` | janela, navegação (voltar/avançar/acima/home, caminho editável, trilha clicável), barra lateral, lista/ícones, menu de contexto, atalhos, tarefas longas | VM |
| `dialogs.rs` | nome (criar/renomear), confirmação, mensagens, propriedades, sobre | VM |
| `fsops.rs` | copiar, mover, renomear, criar, lixeira, excluir, buscar — **sem GTK** | 9 |
| `listing.rs` | listar via GIO (`enumerate_children`), ordenar, formatar | 3 |
| `trash.rs` | lixeira FreeDesktop: listar, restaurar, esvaziar (sem gvfs) | 3 |

## Como o GIO trabalha com arquivos (educacional)
`gio::File` é um *caminho abstrato* (local, `trash:`, `sftp:`…); operações
pedem atributos por nome (`standard::content-type`, `unix::mode`,
`owner::user`, `time::modified`) e devolvem um `FileInfo`. O tipo MIME vem do
banco *shared-mime-info* (nome + conteúdo); o ícone vem do tema. `File::copy`
aceita um `Cancellable` e um callback de progresso em bytes; `File::trash`
implementa a especificação de lixeira FreeDesktop; `AppInfo::launch_default_for_uri`
abre com o aplicativo padrão de `mimeapps.list`.

## Permissões Linux (educacional)
Cada arquivo tem dono, grupo e bits `rwx` para dono/grupo/outros (`755` =
`rwxr-xr-x`). Numa **pasta**, `r` lista, `w` cria/apaga/renomeia dentro e `x`
permite entrar. O kernel aplica as regras em toda chamada; o Files só mostra o
erro (ex.: `/root` → "permissão negada") e mantém a pasta atual.

## Copiar e mover com segurança
| Regra | Onde |
|---|---|
| Nunca sobrescreve: destino existente vira "nome (2).ext" | `fsops::unique_dest` |
| Recursão não segue links simbólicos (o link é recriado) | `copy_one`, `search`, `total_size` |
| Cancelamento cooperativo; o item parcial é removido | `copy_items` |
| Mover no mesmo FS = `rename` atômico; entre FS (`EXDEV`) = copiar tudo → só então apagar a origem | `move_items` |
| Copiar/mover uma pasta para dentro dela mesma é recusado | `copy_items`, `move_items` |
| `/`, `/usr`, `/etc`, `/home`, a própria home, `~/.config`… são protegidos | `is_protected` |
| Exclusão permanente só com confirmação explícita | `ui::delete_permanently` |
| Programas e scripts nunca são executados ao "abrir" | `listing::is_executable_type` |
| Não roda como root (sai com mensagem) | `main::is_root` |
| Sucesso só é anunciado quando a operação retorna Ok | `ui::run_task` |
Erros de I/O (sem espaço, volume removido, somente leitura, permissão) viram
mensagens legíveis (`fsops::describe`).

## Operações longas
`run_task` executa a função numa thread com um `gio::Cancellable`; o progresso
chega por canal (`mpsc`) e um temporizador de 100 ms da GLib atualiza a barra.
A janela continua responsiva; **Esc** ou "Cancelar" cancelam.

## Limitações
- Área de transferência própria (Ctrl+C/X/V entre janelas do Files); para
  outros apps só o texto dos caminhos.
- Sem gvfs/udisks na imagem: sem "Recentes", sem montar dispositivos e sem
  protocolos remotos; a barra lateral lista só o que já está montado.
- Conflito de nomes resolvido sempre com renomeação automática (sem perguntar
  "substituir?").
- Busca recursiva limitada a 5000 resultados e sem indexação.
- Descrições de tipo MIME vêm em inglês quando o shared-mime-info não traz
  tradução (ex.: "Folder").
