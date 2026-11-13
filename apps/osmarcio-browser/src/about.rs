//! Janela "Sobre o OSMarcio Browser" e ícone do aplicativo.
//!
//! ```text
//! Window (não modal, 600x560)
//! └── root (vertical)
//!     ├── BrowserView: página interna (data: URL) com ícone, versões e licenças
//!     │   — texto selecionável; "Copiar informações" copia tudo
//!     ├── status (label): estado da verificação de atualizações
//!     └── botões: [Verificar atualizações] [Copiar informações] [Fechar]
//! ```
//!
//! A verificação roda numa thread comum (dpkg/apt-cache/curl podem demorar) e
//! devolve o resultado à thread de UI com `post_task` — a janela nunca trava.
//! Fechamento segue o mesmo protocolo da janela principal: `can_close` só
//! autoriza depois de `try_close_browser()` do BrowserView retornar 1.

use crate::version::VersionInfo;
use cef::*;
use osmarcio_core::web;
use std::cell::RefCell;

pub const ICON_16: &[u8] =
    include_bytes!("../assets/icons/hicolor/16x16/apps/osmarcio-browser.png");
pub const ICON_32: &[u8] =
    include_bytes!("../assets/icons/hicolor/32x32/apps/osmarcio-browser.png");
pub const ICON_128: &[u8] =
    include_bytes!("../assets/icons/hicolor/128x128/apps/osmarcio-browser.png");
pub const ICON_256: &[u8] =
    include_bytes!("../assets/icons/hicolor/256x256/apps/osmarcio-browser.png");

/// Imagem do CEF com várias resoluções do ícone (escala 1x, 2x).
pub fn icon_image(reps: &[(f32, &[u8])]) -> Option<Image> {
    let img = image_create()?;
    for (scale, png) in reps {
        img.add_png(*scale, Some(png));
    }
    Some(img)
}

/// Aplica o ícone OSMarcio à janela (barra de título, barra de tarefas, Alt+Tab).
pub fn apply_icons(w: &Window) {
    if let Some(mut i) = icon_image(&[(1.0, ICON_16), (2.0, ICON_32)]) {
        w.set_window_icon(Some(&mut i));
    }
    if let Some(mut i) = icon_image(&[(1.0, ICON_128), (2.0, ICON_256)]) {
        w.set_window_app_icon(Some(&mut i));
    }
}

/// WM_CLASS / app_id pedidos ao CEF. Limitação verificada (CEF 156 + cef-rs
/// 156.4.0, X11): o callback é chamado, mas o WM_CLASS não aparece na janela
/// — ver docs/browser/OSMarcioBrowser_Icons.md. O ícone na barra de tarefas e
/// no Alt+Tab vem de _NET_WM_ICON (apply_icons), que funciona.
pub fn linux_props(p: Option<&mut LinuxWindowProperties>) -> i32 {
    if let Some(p) = p {
        p.wm_class_name = CefString::from("osmarcio-browser");
        p.wm_class_class = CefString::from("osmarcio-browser");
        p.wayland_app_id = CefString::from("osmarcio-browser");
        return 1;
    }
    0
}

const CHECK: i32 = 1;
const COPY: i32 = 2;
const CLOSE: i32 = 3;

struct AboutUi {
    window: Window,
    view: BrowserView,
    status: LabelButton,
    check: LabelButton,
}

thread_local! {
    static ABOUT: RefCell<Option<AboutUi>> = const { RefCell::new(None) };
}

fn cs(s: &str) -> CefString {
    CefString::from(s)
}

pub fn is_closed() -> bool {
    ABOUT.with(|a| a.borrow().is_none())
}

/// Abre o Sobre (ou traz para frente se já estiver aberto).
pub fn open() {
    if let Some(w) = ABOUT.with(|a| a.borrow().as_ref().map(|u| u.window.clone())) {
        w.activate();
        return;
    }
    let mut d = AboutWindow::new();
    let _ = window_create_top_level(Some(&mut d));
}

/// Pede o fechamento (usado quando a janela principal fecha).
pub fn close() {
    if let Some(w) = ABOUT.with(|a| a.borrow().as_ref().map(|u| u.window.clone())) {
        w.close();
    }
}

fn page(v: &VersionInfo) -> String {
    let rows: String = v
        .rows()
        .iter()
        .map(|(k, val)| {
            format!(
                "<tr><th>{}</th><td>{}</td></tr>",
                web::html_escape(k),
                web::html_escape(val)
            )
        })
        .collect();
    let icon = web::base64(ICON_128);
    format!(
        r#"<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><title>Sobre o OSMarcio Browser</title>
<meta name="color-scheme" content="light dark"><style>
body{{font-family:"DejaVu Sans",sans-serif;margin:20px 28px;font-size:14px}}
header{{display:flex;gap:18px;align-items:center}} h1{{font-size:22px;margin:0}} .sub{{opacity:.75}}
table{{border-collapse:collapse;margin:16px 0;width:100%}} th{{text-align:left;font-weight:600;padding:3px 14px 3px 0;white-space:nowrap}}
td{{font-family:"DejaVu Sans Mono",monospace;font-size:13px}} p{{margin:6px 0;font-size:12.5px;opacity:.85}}
</style></head><body>
<header><img src="data:image/png;base64,{icon}" width="80" height="80" alt="ícone OSMarcio Browser">
<div><h1>OSMarcio Browser</h1><div class="sub">Navegador do OSMarcio Linux · Rust + Chromium Embedded Framework</div></div></header>
<table>{rows}</table>
<p><b>Licenças:</b> código do OSMarcio Browser — GPL-3.0-or-later. Motor Chromium/CEF — BSD-3-Clause e
licenças de terceiros listadas em <code>CREDITS.html</code> e <code>LICENSE.txt</code> no diretório do aplicativo.</p>
<p><b>Projeto:</b> OSMarcio Linux — distribuição educacional baseada em Debian 13. Ícone original do projeto.</p>
<p><b>Atualizações:</b> o Chromium faz parte do pacote <code>osmarcio-browser</code>; uma versão nova do
motor só chega como nova versão homologada do pacote, por canal assinado (APT).</p>
</body></html>"#
    )
}

wrap_window_delegate! {
    struct AboutWindow {}

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 600, height: 560 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            if let Some(w) = window {
                build(w.clone());
            }
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            let host = ABOUT.with(|a| a.borrow().as_ref().and_then(|u| u.view.browser()).and_then(|b| b.host()));
            match host {
                Some(h) => h.try_close_browser(),
                None => 1,
            }
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            ABOUT.with(|a| a.borrow_mut().take());
            log::info!("osmb: sobre fechado");
            crate::ui::maybe_quit();
        }

        fn linux_window_properties(&self, _window: Option<&mut Window>, properties: Option<&mut LinuxWindowProperties>) -> i32 {
            linux_props(properties)
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

wrap_browser_view_delegate! {
    struct AboutView {}

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

wrap_client! {
    struct AboutClient;

    impl Client {}
}

wrap_button_delegate! {
    struct AboutButton {
        cmd: i32,
    }

    impl ViewDelegate {}

    impl ButtonDelegate {
        fn on_button_pressed(&self, _button: Option<&mut Button>) {
            pressed(self.cmd);
        }
    }
}

wrap_task! {
    struct UpdateResult {
        msg: String,
    }

    impl Task {
        fn execute(&self) {
            log::info!("osmb: atualizacoes: {}", self.msg);
            if let Some((status, check)) = ABOUT.with(|a| a.borrow().as_ref().map(|u| (u.status.clone(), u.check.clone()))) {
                status.set_text(Some(&cs(&self.msg)));
                check.set_enabled(1);
            }
        }
    }
}

fn button(text: &str, cmd: i32) -> LabelButton {
    let mut d = AboutButton::new(cmd);
    label_button_create(Some(&mut d), Some(&cs(text))).expect("label_button_create")
}

fn build(window: Window) {
    let info = VersionInfo::collect();
    let root = panel_create(None).expect("panel");
    let mut settings = crate::ui::vbox_settings();
    settings.inside_border_horizontal_spacing = 12;
    settings.inside_border_vertical_spacing = 10;
    settings.between_child_spacing = 8;
    let layout = root.set_to_box_layout(Some(&settings));

    let mut client = Some(AboutClient::new());
    let mut bvd = AboutView::new();
    let url = web::data_url(&page(&info));
    let view = browser_view_create(
        client.as_mut(),
        Some(&cs(&url)),
        Some(&BrowserSettings::default()),
        None,
        None,
        Some(&mut bvd),
    )
    .expect("browser_view_create");
    let status = button("Clique em “Verificar atualizações”.", 0);
    status.set_enabled(0);
    let buttons = panel_create(None).expect("panel");
    buttons.set_to_box_layout(Some(&BoxLayoutSettings {
        horizontal: 1,
        between_child_spacing: 8,
        ..Default::default()
    }));
    let check = button("Verificar atualizações", CHECK);
    for b in [
        &check,
        &button("Copiar informações", COPY),
        &button("Fechar", CLOSE),
    ] {
        let mut v: View = b.into();
        buttons.add_child_view(Some(&mut v));
    }
    let mut vv: View = (&view).into();
    root.add_child_view(Some(&mut vv));
    let mut sv: View = (&status).into();
    root.add_child_view(Some(&mut sv));
    let mut bv: View = (&buttons).into();
    root.add_child_view(Some(&mut bv));
    if let Some(l) = layout {
        l.set_flex_for_view(Some(&mut vv), 1);
    }
    let mut rv: View = (&root).into();
    window.add_child_view(Some(&mut rv));
    window.set_title(Some(&cs("Sobre o OSMarcio Browser")));
    apply_icons(&window);
    ABOUT.with(|a| {
        *a.borrow_mut() = Some(AboutUi {
            window: window.clone(),
            view,
            status,
            check,
        })
    });
    window.show();
    log::info!(
        "osmb: sobre aberto browser={} chromium={} cef={} cef_compilado={} cef_rs={} arch={} sistema={} {} runtime_confere={}",
        info.browser,
        info.chromium,
        info.cef,
        info.cef_compiled,
        info.cef_rs,
        info.arch,
        info.os,
        info.os_version,
        info.runtime_matches_build()
    );
}

fn pressed(cmd: i32) {
    let Some((window, view, status, check)) = ABOUT.with(|a| {
        a.borrow().as_ref().map(|u| {
            (
                u.window.clone(),
                u.view.clone(),
                u.status.clone(),
                u.check.clone(),
            )
        })
    }) else {
        return;
    };
    match cmd {
        CHECK => {
            status.set_text(Some(&cs("Verificando atualizações…")));
            check.set_enabled(0);
            let info = VersionInfo::collect();
            let (running, arch) = (info.browser.clone(), info.deb_arch().to_string());
            std::thread::spawn(move || {
                let msg = crate::updates::check(&running, &arch).message();
                let mut t = UpdateResult::new(msg);
                post_task(ThreadId::UI, Some(&mut t));
            });
        }
        COPY => {
            if let Some(f) = view.browser().and_then(|b| b.main_frame()) {
                f.select_all();
                f.copy();
                status.set_text(Some(&cs(
                    "Informações de versão copiadas para a área de transferência.",
                )));
            }
        }
        CLOSE => window.close(),
        _ => {}
    }
}
