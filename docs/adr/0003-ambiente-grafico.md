# ADR 0003 — Ambiente gráfico: labwc (Wayland) + GTK4, com XWayland

- **Status:** Proposta (decisão a confirmar com protótipo na Fase 6)

## Contexto

Precisamos de um desktop leve, com painel, menu, Alt+Tab, notificações,
login gráfico etc., que funcione em QEMU/UTM (virtio-gpu), VirtualBox
(VMSVGA) e RK3399 (Mali-T860 via Panfrost), e que hospede bem os apps
OSMarcio (GTK4) e o browser CEF.

Não vamos escrever compositor, servidor gráfico ou window manager próprios.

## Alternativas

| | labwc + componentes | XFCE (X11) | LXQt |
|---|---|---|---|
| Peso (RAM ociosa, estimativa) | muito baixo | baixo | baixo |
| Wayland nativo | ✅ | ⚠️ (Xfce 4.20 parcial) | ⚠️ |
| Toolkit dos apps OSMarcio (GTK4) | natural | natural (GTK3) | Qt — destoa |
| CEF | via XWayland/Ozone | X11 nativo | X11 |
| Identidade visual própria | fácil (tema openbox-like + waybar/sfwbar) | média | média |
| Pronto "de fábrica" (painel, menu, configurações) | ❌ montar peças | ✅ | ✅ |

## Proposta

- **Compositor:** `labwc` (Debian 13), Wayland, com `XWayland` para
  compatibilidade (CEF, apps X11 legados).
- **Painel/menu:** `sfwbar` ou `waybar` + `fuzzel` (avaliar na Fase 6).
- **Login:** `greetd` + `gtkgreet` (leve) ou `lightdm` (mais maduro).
- **Notificações:** `mako`; **papel de parede:** `swaybg`.
- **Plano B:** se a integração exigir peças demais para os critérios do
  prompt (configurações, resolução, desligar/reiniciar), adotar **XFCE**
  com tema OSMarcio — mais pesado, porém completo.

## Riscos a validar no protótipo

1. virtio-gpu em UTM com e sem aceleração (`virglrenderer`).
2. VirtualBox VMSVGA sob Wayland (historicamente instável; fallback X11).
3. CEF sob XWayland vs. Ozone/Wayland.
4. Mali-T860/Panfrost na ROCK Pi 4A.
