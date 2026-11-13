# OSMarcio Terminal — abas maiores (0.3.0)

**Estado: Implementado e testado (ARM64, QEMU/HVF + Xvfb).**

## Problema e causa
Rótulo da aba com apenas `max_width_chars(24)`: sem largura mínima, o
`GtkNotebook` encolhia os títulos até "…" com várias abas; padding do tema
pequeno.

## Solução (`apps/osmarcio-terminal/src/window.rs`)
- `width_chars(18)` (mínimo ≈160 px), `max_width_chars(28)`, reticências,
  tooltip com o título completo (atualizado com o título do shell).
- CSS restrito às abas (`notebook.osm-tabs`): padding 5×14 px, altura mínima
  30 px, aba ativa em negrito, ✕ com 22 px. Fonte do terminal inalterada.
- Sem largura fixa de janela; faixa de abas rolável (setas) com muitas abas.
- Ícone próprio `osmarcio-terminal` (janela, painel, menu, pacote).
Detalhes: [docs/terminal/OSMarcioTerminal_UI.md](../docs/terminal/OSMarcioTerminal_UI.md).

## Testes
| Teste | Resultado |
|---|---|
| `make apps-test` (Xvfb): 10 cenários de abas/sessões | 10/10 |
| `make test-desktop-apps` — terminal-1/5/10/fim | 10/10: última aba sem ✕ e protegida (Ctrl+Shift+W), 5 e 10 abas, fechamento 10→1, maximizar 1280×763, redimensionar 520×300, fechar |
| Capturas | `reports/tests/screenshots/desktop-apps-arm64-*-0{1,2,3}-terminal-*.png` |

Antes (captura do diagnóstico): títulos cortados após poucos caracteres. Depois:
abas de ~205 px com "osmarcio@osmarcio-…", setas de rolagem com 5+ abas.

## Limitações
Tema claro × escuro e escala 2× não testados; os títulos padrão do bash
(`usuário@host: dir`) continuam longos — a aba mostra o início e o tooltip o resto.
