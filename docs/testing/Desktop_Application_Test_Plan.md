# Plano de testes dos aplicativos do desktop (0.3.0)

Princípio: nada é declarado funcionando sem execução. Testes de GUI usam a
imagem real numa VM QEMU/HVF (mesma máquina virtual do UTM), login real no
LightDM, entrada por XTest (`xdotool`) ou teclas do QEMU, e conferem o
resultado no sistema de arquivos, nos logs dos apps e em capturas de tela.

| Nível | Comando | Cobertura |
|---|---|---|
| Lint | `make apps-lint` | rustfmt + clippy `-D warnings` no workspace |
| Unitário | `make apps-test` | core 30 · Files 15 (operações reais em /tmp) · Campo Minado 18 · Terminal 3 + GUI Xvfb 10 |
| Boot | `make test-boot` | 2 boots, rede, APT, persistência, LightDM, pacotes/ícones/.desktop/MIME dos 5 apps |
| Desktop | `make test-desktop` | login gráfico com troca de senha, XFCE, Terminal executa comando, Browser renderiza HTTPS |
| Apps | `make test-desktop-apps` | tabelas abaixo (+ 11 capturas) |
| Browser | `make test-browser-hotfix` | fechamento, ícone, Sobre, atualizações (0.2.1) |

## `test-desktop-apps` — cenários
**Terminal:** 1 aba sem ✕ · Ctrl+Shift+W na última aba não fecha · 5 e 10 abas
(captura) · fechamento individual 10→1 · maximizar · redimensionar · fechar.

**Browser:** HTTPS · ação Maximizar anunciada · Alt+F10 = área útil · evento de
tamanho do CEF · captura com conteúdo além de 1100 px · restaurar ao tamanho
anterior · nova aba carregando maximizada · troca de aba e rolagem · F11 entra e
sai (volta maximizada) · redimensionar · Ctrl+Q sem órfãos · reabrir.

**Files** (em `~/osmf-teste`, criado e removido pelo teste): listar (ocultos
escondidos) · nova pasta · copiar (origem preservada) · mover · renomear com
Unicode · lixeira · restaurar · cancelar cópia de 2 GiB (sem parcial) · cópia
de 2 GiB completa com barra de progresso (tamanho idêntico) · permissão negada
em /root · propriedades · busca · Ctrl+H · abrir texto no editor padrão ·
script executável **não** executado · visão de ícones · exclusão permanente com
confirmação (não vai à lixeira) · fechar.

**Campo Minado** (semente fixa): primeiro clique seguro com 16 minas em 5×5 ·
vitória · 10 minas exatas no Fácil · bandeira marcar/desmarcar e contador ·
derrota numa mina conhecida e parada do temporizador · F2 reinicia · vitória
completa no Fácil pelo teclado · recorde salvo · fechar.

## Fora do alcance desta etapa (não declarado como testado)
x86_64 (QEMU AMD64/VirtualBox): não há build amd64 da distribuição · ROCK Pi 4A:
sem hardware · escala 2× e outras resoluções · tema claro/escuro alternados ·
clique do mouse no tabuleiro do Campo Minado (testado só teclado; o clique
direito é coberto pelo mesmo `toggle_flag`) · app UTM com a 0.3.0 (QEMU/HVF sim).
