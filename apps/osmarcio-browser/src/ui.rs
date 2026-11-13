//! Interface do OSMarcio Browser, construída em Rust com o CEF Views:
//!
//! ```text
//! Window
//! └── root (vertical)
//!     ├── tabstrip (horizontal): [aba 1] [aba 2] … [+]
//!     ├── toolbar  (horizontal): [◀][▶][⟳][⌂][🔒 HTTPS][ barra de endereços ][★][✕]
//!     ├── content  (vertical):   BrowserView da aba ativa (as demais ocultas)
//!     └── status   (label):      mensagens (downloads, erros da barra)
//! ```
//!
//! Todo o estado vive na thread de UI do CEF (`thread_local!`). Regra para
//! evitar reentrância: copiar o que for preciso do estado, soltar o
//! `borrow`, e só então chamar o CEF.

use cef::*;
use osmarcio_core::web::{self, Entry, EntryFile, Security, Target};
use std::cell::RefCell;
use std::path::PathBuf;

// Comandos (ids de aceleradores e botões).
const NEW_TAB: i32 = 1;
const CLOSE_TAB: i32 = 2;
const NEXT_TAB: i32 = 3;
const PREV_TAB: i32 = 4;
const FOCUS_URL: i32 = 5;
const RELOAD: i32 = 6;
const BACK: i32 = 7;
const FORWARD: i32 = 8;
const HOME: i32 = 9;
const ZOOM_IN: i32 = 10;
const ZOOM_OUT: i32 = 11;
const ZOOM_RESET: i32 = 12;
const FULLSCREEN: i32 = 13;
const BOOKMARK: i32 = 14;
const QUIT: i32 = 15;
const CLOSE_OTHERS: i32 = 16;
const ABOUT: i32 = 17;

// Códigos de tecla (Windows virtual-key codes, usados pelo CEF em todas as plataformas).
const VK_TAB: i32 = 0x09;
const VK_LEFT: i32 = 0x25;
const VK_RIGHT: i32 = 0x27;
const VK_F1: i32 = 0x70;
const VK_F5: i32 = 0x74;
const VK_F11: i32 = 0x7A;
const VK_OEM_PLUS: i32 = 0xBB;
const VK_OEM_MINUS: i32 = 0xBD;

struct Tab {
    id: u32,
    /// Id do browser CEF (definido em on_after_created). Guardado porque
    /// `view.browser()` deixa de responder durante a destruição da janela.
    browser_id: Option<i32>,
    view: BrowserView,
    button: LabelButton,
    title: String,
    url: String,
    home: bool,
}

struct Ui {
    window: Window,
    tabstrip: Panel,
    content: Panel,
    url_field: Textfield,
    security: LabelButton,
    back: LabelButton,
    forward: LabelButton,
    reload: LabelButton,
    status: LabelButton,
}

#[derive(Default)]
struct State {
    ui: Option<Ui>,
    tabs: Vec<Tab>,
    active: usize,
    next_id: u32,
    /// Fechamento da janela em andamento (X, Alt+F4, Ctrl+Q, menu, última aba).
    closing: bool,
    /// A janela principal já foi destruída pelo CEF.
    window_destroyed: bool,
    /// quit_message_loop() já foi chamado.
    quitting: bool,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

pub fn selftest() -> bool {
    std::env::var("OSMARCIO_BROWSER_SELFTEST").is_ok_and(|v| v == "1")
}

fn cs(s: &str) -> CefString {
    CefString::from(s)
}

// ----------------------------------------------------------------- delegates

wrap_window_delegate! {
    struct OsmWindow {}

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1100, height: 760 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            if let Some(w) = window {
                build_ui(w.clone());
            }
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            // Protocolo do CEF Views para fechar uma janela com browsers:
            // try_close_browser() em cada aba. Retorna 0 enquanto a aba ainda
            // precisa de um ciclo (beforeunload, DoClose) — o CEF então pede o
            // fechamento da janela de novo e, na 2ª passada, retorna 1.
            // (O código antigo chamava close_browser(0) e sempre retornava 0:
            // o browser esperava a janela e a janela esperava o browser —
            // o botão X "não fazia nada".)
            let hosts: Vec<BrowserHost> = with(|s| {
                s.closing = true;
                s.tabs.iter().filter_map(|t| t.view.browser().and_then(|b| b.host())).collect()
            });
            crate::about::close();
            let mut ready = true;
            for h in &hosts {
                if h.try_close_browser() != 1 {
                    ready = false;
                }
            }
            log::info!("osmb: fechar janela solicitado abas={} pode_fechar={ready}", hosts.len());
            ready as i32
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            with(|s| {
                s.window_destroyed = true;
                s.ui = None;
            });
            log::info!("osmb: janela destruída");
            maybe_quit();
        }

        fn linux_window_properties(&self, _window: Option<&mut Window>, properties: Option<&mut LinuxWindowProperties>) -> i32 {
            crate::about::linux_props(properties)
        }

        fn on_accelerator(&self, _window: Option<&mut Window>, command_id: i32) -> i32 {
            log::info!("osmb: atalho cmd={command_id}");
            command(command_id);
            1
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

wrap_browser_view_delegate! {
    struct OsmBrowserView {}

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            // Estilo "Alloy": só a área da página; toda a interface é nossa (Rust).
            RuntimeStyle::ALLOY
        }
    }
}

wrap_button_delegate! {
    struct ToolButton {
        cmd: i32,
    }

    impl ViewDelegate {}

    impl ButtonDelegate {
        fn on_button_pressed(&self, _button: Option<&mut Button>) {
            command(self.cmd);
        }
    }
}

wrap_button_delegate! {
    struct TabButton {
        tab_id: u32,
    }

    impl ViewDelegate {}

    impl ButtonDelegate {
        fn on_button_pressed(&self, _button: Option<&mut Button>) {
            let id = self.tab_id;
            if let Some(i) = with(|s| s.tabs.iter().position(|t| t.id == id)) {
                activate(i);
            }
        }
    }
}

wrap_textfield_delegate! {
    struct UrlField {}

    impl ViewDelegate {}

    impl TextfieldDelegate {
        fn on_key_event(&self, textfield: Option<&mut Textfield>, event: Option<&KeyEvent>) -> i32 {
            let (Some(tf), Some(ev)) = (textfield, event) else { return 0 };
            if ev.type_ == KeyEventType::RAWKEYDOWN && ev.windows_key_code == 0x0D {
                let text = CefString::from(&tf.text()).to_string();
                navigate_input(&text);
                return 1;
            }
            0
        }
    }
}

wrap_task! {
    struct NewTabTask {
        url: String,
    }

    impl Task {
        fn execute(&self) {
            new_tab(Some(self.url.clone()));
        }
    }
}

// --------------------------------------------------------------- construção

wrap_menu_button_delegate! {
    struct MainMenu {}

    impl ViewDelegate {}

    impl ButtonDelegate {
        fn on_button_pressed(&self, _button: Option<&mut Button>) {}
    }

    impl MenuButtonDelegate {
        fn on_menu_button_pressed(&self, menu_button: Option<&mut MenuButton>, screen_point: Option<&Point>, _lock: Option<&mut MenuButtonPressedLock>) {
            let Some(mb) = menu_button else { return };
            let mut d = MenuCommands::new();
            let Some(mut model) = menu_model_create(Some(&mut d)) else { return };
            model.add_item(NEW_TAB, Some(&cs("Nova aba\tCtrl+T")));
            model.add_item(CLOSE_TAB, Some(&cs("Fechar aba\tCtrl+W")));
            model.add_separator();
            model.add_item(ABOUT, Some(&cs("Sobre o OSMarcio Browser\tF1")));
            model.add_separator();
            model.add_item(QUIT, Some(&cs("Fechar janela\tCtrl+Q")));
            log::info!("osmb: menu aberto");
            mb.show_menu(Some(&mut model), screen_point, MenuAnchorPosition::TOPRIGHT);
        }
    }
}

wrap_menu_model_delegate! {
    struct MenuCommands {}

    impl MenuModelDelegate {
        fn execute_command(&self, _menu_model: Option<&mut MenuModel>, command_id: i32, _event_flags: EventFlags) {
            log::info!("osmb: menu cmd={command_id}");
            command(command_id);
        }
    }
}

pub fn create_main_window() {
    let mut d = OsmWindow::new();
    window_create_top_level(Some(&mut d));
}

fn button(text: &str, tip: &str, cmd: i32) -> LabelButton {
    let mut d = ToolButton::new(cmd);
    let b = label_button_create(Some(&mut d), Some(&cs(text))).expect("label_button_create");
    b.set_tooltip_text(Some(&cs(tip)));
    b
}

fn hbox(panel: &Panel, spacing: i32) -> Option<BoxLayout> {
    panel.set_to_box_layout(Some(&BoxLayoutSettings {
        horizontal: 1,
        between_child_spacing: spacing,
        inside_border_insets: Insets {
            top: 2,
            left: 4,
            bottom: 2,
            right: 4,
        },
        cross_axis_alignment: AxisAlignment::CENTER,
        ..Default::default()
    }))
}

fn add(panel: &Panel, v: impl Into<View>) -> View {
    let mut v: View = v.into();
    panel.add_child_view(Some(&mut v));
    v
}

/// Layout vertical com os filhos esticados na largura (cross axis = STRETCH).
/// `BoxLayoutSettings::default()` do cef-rs zera a struct, o que dá
/// `cross_axis_alignment = START`: a BrowserView ficava com a largura
/// preferida (0 px) e a página carregava mas não aparecia.
pub(crate) fn vbox_settings() -> BoxLayoutSettings {
    BoxLayoutSettings {
        horizontal: 0,
        cross_axis_alignment: AxisAlignment::STRETCH,
        ..Default::default()
    }
}

fn build_ui(window: Window) {
    let root = panel_create(None).expect("panel");
    root.set_to_box_layout(Some(&vbox_settings()));

    let tabstrip = panel_create(None).expect("panel");
    let tab_layout = hbox(&tabstrip, 2);
    let plus = button("+", "Nova aba (Ctrl+T)", NEW_TAB);
    add(&tabstrip, &plus);
    let _ = tab_layout;

    let toolbar = panel_create(None).expect("panel");
    let tl = hbox(&toolbar, 4);
    let back = button("◀", "Voltar (Alt+←)", BACK);
    let forward = button("▶", "Avançar (Alt+→)", FORWARD);
    let reload = button("⟳", "Recarregar (F5)", RELOAD);
    let home = button("⌂", "Página inicial", HOME);
    let security = button("OSMarcio", "Segurança da conexão", 0);
    let mut uf = UrlField::new();
    let url_field = textfield_create(Some(&mut uf)).expect("textfield");
    url_field.set_placeholder_text(Some(&cs("Digite um endereço ou pesquise")));
    let star = button("★", "Adicionar aos favoritos (Ctrl+D)", BOOKMARK);
    let close = button("✕", "Fechar aba (Ctrl+W)", CLOSE_TAB);
    for b in [&back, &forward, &reload, &home, &security] {
        add(&toolbar, b);
    }
    let mut url_view = add(&toolbar, &url_field);
    add(&toolbar, &star);
    add(&toolbar, &close);
    let mut mb = MainMenu::new();
    let menu = menu_button_create(Some(&mut mb), Some(&cs("☰"))).expect("menu_button_create");
    menu.set_tooltip_text(Some(&cs("Menu do OSMarcio Browser")));
    add(&toolbar, &menu);
    if let Some(tl) = tl {
        tl.set_flex_for_view(Some(&mut url_view), 1);
    }

    let content = panel_create(None).expect("panel");
    content.set_to_box_layout(Some(&vbox_settings()));

    let status = button("", "", 0);
    status.set_enabled(0);

    add(&root, &tabstrip);
    add(&root, &toolbar);
    let mut content_view = add(&root, &content);
    add(&root, &status);
    if let Some(l) = root.set_to_box_layout(Some(&vbox_settings())) {
        l.set_flex_for_view(Some(&mut content_view), 1);
    }

    let mut root_view: View = (&root).into();
    window.add_child_view(Some(&mut root_view));
    window.set_title(Some(&cs("OSMarcio Browser")));
    crate::about::apply_icons(&window);

    // Atalhos de teclado (aceleradores da janela).
    for (cmd, key, shift, ctrl, alt) in [
        (NEW_TAB, b'T' as i32, 0, 1, 0),
        (CLOSE_TAB, b'W' as i32, 0, 1, 0),
        (CLOSE_OTHERS, b'W' as i32, 1, 1, 0),
        (NEXT_TAB, VK_TAB, 0, 1, 0),
        (PREV_TAB, VK_TAB, 1, 1, 0),
        (FOCUS_URL, b'L' as i32, 0, 1, 0),
        (RELOAD, VK_F5, 0, 0, 0),
        (RELOAD, b'R' as i32, 0, 1, 0),
        (BACK, VK_LEFT, 0, 0, 1),
        (FORWARD, VK_RIGHT, 0, 0, 1),
        (ZOOM_IN, VK_OEM_PLUS, 0, 1, 0),
        (ZOOM_OUT, VK_OEM_MINUS, 0, 1, 0),
        (ZOOM_RESET, b'0' as i32, 0, 1, 0),
        (FULLSCREEN, VK_F11, 0, 0, 0),
        (BOOKMARK, b'D' as i32, 0, 1, 0),
        (QUIT, b'Q' as i32, 0, 1, 0),
        (ABOUT, VK_F1, 0, 0, 0),
    ] {
        // high_priority=1: o atalho do navegador vence a página (como no Chrome).
        window.set_accelerator(cmd, key, shift, ctrl, alt, 1);
    }

    with(|s| {
        s.ui = Some(Ui {
            window: window.clone(),
            tabstrip,
            content,
            url_field,
            security,
            back,
            forward,
            reload,
            status,
        })
    });

    let first = std::env::args()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .or_else(|| std::env::args().find_map(|a| a.strip_prefix("--url=").map(String::from)));
    new_tab(first);
    window.show();
    log::info!("osmb: janela criada");
    if std::env::args().any(|a| a == "--about") {
        crate::about::open();
    }
}

// ------------------------------------------------------------------ abas

/// Abre uma aba. `None` = página inicial.
pub fn new_tab(input: Option<String>) {
    let target = input
        .as_deref()
        .map(web::parse_input)
        .unwrap_or(Target::Home);
    let (url, home) = match &target {
        Target::Home => (home_url(), true),
        Target::Rejected(why) => {
            notify_status(why);
            (home_url(), true)
        }
        t => (t.to_url().unwrap_or_else(home_url), false),
    };

    let Some((tabstrip, content)) = with(|s| {
        s.ui.as_ref()
            .map(|u| (u.tabstrip.clone(), u.content.clone()))
    }) else {
        return;
    };
    let id = with(|s| {
        s.next_id += 1;
        s.next_id
    });

    let mut client = Some(crate::client::OsmClient::new());
    let mut bvd = OsmBrowserView::new();
    let settings = BrowserSettings::default();
    let Some(view) = browser_view_create(
        client.as_mut(),
        Some(&cs(&url)),
        Some(&settings),
        None,
        None,
        Some(&mut bvd),
    ) else {
        log::error!("osmb: falha ao criar BrowserView");
        return;
    };
    let mut tb = TabButton::new(id);
    let button = label_button_create(Some(&mut tb), Some(&cs("Nova aba"))).expect("tab button");

    // Insere o botão antes do "+" (último filho da faixa de abas).
    let n = tabstrip.child_view_count() as i32;
    let mut bv: View = (&button).into();
    tabstrip.add_child_view_at(Some(&mut bv), (n - 1).max(0));
    let mut vv: View = (&view).into();
    content.add_child_view(Some(&mut vv));
    if let Some(l) = content.set_to_box_layout(Some(&vbox_settings())) {
        l.set_flex_for_view(Some(&mut vv), 1);
    }

    let idx = with(|s| {
        s.tabs.push(Tab {
            id,
            browser_id: None,
            view,
            button,
            title: String::new(),
            url: url.clone(),
            home,
        });
        s.tabs.len() - 1
    });
    log::info!("osmb: aba aberta id={id} total={}", idx + 1);
    activate(idx);
}

pub fn post_new_tab(url: String) {
    let mut t = NewTabTask::new(url);
    post_task(ThreadId::UI, Some(&mut t));
}

fn activate(idx: usize) {
    let Some((views, active, ui_bits)) = with(|s| {
        if idx >= s.tabs.len() {
            return None;
        }
        s.active = idx;
        let views: Vec<(View, LabelButton, bool)> = s
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| ((&t.view).into(), t.button.clone(), i == idx))
            .collect();
        let t = &s.tabs[idx];
        Some((
            views,
            (t.view.clone(), t.url.clone(), t.home, t.title.clone()),
            s.ui.as_ref().map(|u| (u.content.clone(), u.window.clone())),
        ))
    }) else {
        return;
    };
    for (v, b, on) in &views {
        v.set_visible(*on as i32);
        b.set_state(if *on {
            ButtonState::PRESSED
        } else {
            ButtonState::NORMAL
        });
    }
    if let Some((content, window)) = ui_bits {
        content.layout();
        let title = if active.3.is_empty() {
            "OSMarcio Browser".to_string()
        } else {
            format!("{} — OSMarcio Browser", active.3)
        };
        window.set_title(Some(&cs(&title)));
    }
    show_address(&active.1, active.2);
    active.0.request_focus();
}

fn close_tab_at(idx: usize) {
    let host = with(|s| {
        s.tabs
            .get(idx)
            .and_then(|t| t.view.browser())
            .and_then(|b| b.host())
    });
    if let Some(h) = host {
        // Fecha o browser; o processo renderer é encerrado e on_before_close remove a aba.
        h.close_browser(0);
    }
}

/// Chamado pelo CEF quando o browser de uma aba terminou de fechar.
pub fn on_browser_closed(browser_id: i32) {
    let (removed, ui, remaining, active, closing) = with(|s| {
        let pos = pos_of(s, browser_id);
        let removed = pos.map(|p| s.tabs.remove(p));
        if s.active >= s.tabs.len() && !s.tabs.is_empty() {
            s.active = s.tabs.len() - 1;
        }
        (
            removed,
            s.ui.as_ref()
                .map(|u| (u.tabstrip.clone(), u.content.clone(), u.window.clone())),
            s.tabs.len(),
            s.active,
            s.closing,
        )
    });
    log::info!("osmb: aba fechada browser={browser_id} restantes={remaining}");
    if let (Some(t), Some((strip, content, _))) = (&removed, &ui) {
        let mut b: View = (&t.button).into();
        strip.remove_child_view(Some(&mut b));
        let mut v: View = (&t.view).into();
        content.remove_child_view(Some(&mut v));
        strip.layout();
    }
    if remaining == 0 {
        // Última aba fechada pelo usuário (Ctrl+W): fecha a janela pelo
        // mesmo caminho do botão X. Se a janela já está fechando, nada a fazer.
        if let (false, Some((_, _, w))) = (closing, &ui) {
            with(|s| s.closing = true);
            w.close();
        }
        maybe_quit();
    } else if !closing {
        activate(active);
    }
}

/// Encerra o loop de mensagens do CEF somente quando a janela principal foi
/// destruída, todas as abas terminaram de fechar e o "Sobre" está fechado —
/// assim `shutdown()` roda sem browsers vivos e não sobram subprocessos.
pub fn maybe_quit() {
    let go = with(|s| {
        if s.window_destroyed {
            // Abas cujo browser já não existe (destruído junto com a janela).
            s.tabs
                .retain(|t| t.view.browser().is_some() && t.browser_id.is_some());
        }
        let go = s.window_destroyed && s.tabs.is_empty() && !s.quitting;
        if go {
            s.quitting = true;
        }
        go
    });
    if go && crate::about::is_closed() {
        log::info!("osmb: encerrando loop de mensagens");
        quit_message_loop();
    } else if go {
        with(|s| s.quitting = false);
    }
}

// --------------------------------------------------------------- navegação

fn home_url() -> String {
    let bm = EntryFile::in_data_dir("bookmarks")
        .map(|f| f.recent(20))
        .unwrap_or_default();
    let hist = EntryFile::in_data_dir("history")
        .map(|f| f.recent(12))
        .unwrap_or_default();
    web::data_url(&web::home_page(osmarcio_core::VERSION, &bm, &hist))
}

fn active_browser() -> Option<Browser> {
    with(|s| s.tabs.get(s.active).and_then(|t| t.view.browser()))
}

fn navigate_input(text: &str) {
    let target = web::parse_input(text);
    log::info!("osmb: barra de endereços -> {target:?}");
    let (url, home) = match &target {
        Target::Rejected(why) => {
            notify_status(why);
            return;
        }
        Target::Home => (home_url(), true),
        t => (t.to_url().unwrap_or_default(), false),
    };
    with(|s| {
        let a = s.active;
        if let Some(t) = s.tabs.get_mut(a) {
            t.home = home;
        }
    });
    if let Some(frame) = active_browser().and_then(|b| b.main_frame()) {
        frame.load_url(Some(&cs(&url)));
    }
    if let Some(h) = active_browser().and_then(|b| b.host()) {
        h.set_focus(1);
    }
}

fn show_address(url: &str, home: bool) {
    let Some((field, sec)) = with(|s| {
        s.ui.as_ref()
            .map(|u| (u.url_field.clone(), u.security.clone()))
    }) else {
        return;
    };
    let shown = if home {
        web::HOME_URL.to_string()
    } else if url.starts_with("data:") {
        "(página interna do OSMarcio)".to_string()
    } else {
        url.to_string()
    };
    field.set_text(Some(&cs(&shown)));
    sec.set_text(Some(&cs(
        Security::of(if home { "data:" } else { url }).label()
    )));
}

pub fn command(cmd: i32) {
    let n = with(|s| s.tabs.len());
    let cur = with(|s| s.active);
    match cmd {
        NEW_TAB => new_tab(None),
        CLOSE_TAB => close_tab_at(cur),
        CLOSE_OTHERS => {
            for i in (0..n).rev().filter(|&i| i != cur) {
                close_tab_at(i);
            }
        }
        NEXT_TAB if n > 0 => activate((cur + 1) % n),
        PREV_TAB if n > 0 => activate((cur + n - 1) % n),
        FOCUS_URL => {
            if let Some(f) = with(|s| s.ui.as_ref().map(|u| u.url_field.clone())) {
                f.request_focus();
                f.select_all(1);
            }
        }
        RELOAD => {
            if let Some(b) = active_browser() {
                b.reload();
            }
        }
        BACK => {
            if let Some(b) = active_browser() {
                b.go_back();
            }
        }
        FORWARD => {
            if let Some(b) = active_browser() {
                b.go_forward();
            }
        }
        HOME => navigate_input(web::HOME_URL),
        ZOOM_IN | ZOOM_OUT | ZOOM_RESET => {
            if let Some(h) = active_browser().and_then(|b| b.host()) {
                let z = match cmd {
                    ZOOM_IN => (h.zoom_level() + 0.5).min(5.0),
                    ZOOM_OUT => (h.zoom_level() - 0.5).max(-5.0),
                    _ => 0.0,
                };
                h.set_zoom_level(z);
                log::info!("osmb: zoom={z}");
            }
        }
        FULLSCREEN => {
            if let Some(w) = with(|s| s.ui.as_ref().map(|u| u.window.clone())) {
                let fs = w.is_fullscreen() != 1;
                w.set_fullscreen(fs as i32);
            }
        }
        BOOKMARK => {
            let cur_entry = with(|s| {
                s.tabs
                    .get(s.active)
                    .map(|t| (t.url.clone(), t.title.clone(), t.home))
            });
            if let (Some((url, title, false)), Some(f)) =
                (cur_entry, EntryFile::in_data_dir("bookmarks"))
                && url.starts_with("http")
                && !f.contains(&url)
            {
                let _ = f.append(&Entry {
                    url: url.clone(),
                    title,
                });
                notify_status(&format!("Favorito adicionado: {url}"));
                log::info!("osmb: favorito {url}");
            }
        }
        ABOUT => crate::about::open(),
        QUIT => {
            if let Some(w) = with(|s| s.ui.as_ref().map(|u| u.window.clone())) {
                w.close();
            }
        }
        _ => {}
    }
}

// ------------------------------------------------- eventos vindos do client

fn pos_of(s: &State, browser_id: i32) -> Option<usize> {
    s.tabs.iter().position(|t| {
        t.browser_id == Some(browser_id)
            || t.view.browser().map(|b| b.identifier()) == Some(browser_id)
    })
}

fn tab_index(browser_id: i32) -> Option<usize> {
    with(|s| pos_of(s, browser_id))
}

pub fn on_browser_created(browser_id: i32) {
    with(|s| {
        // A BrowserView pode ainda não expor o browser neste instante; as abas
        // são criadas uma de cada vez, então a última sem id é a dona dele.
        let i = s
            .tabs
            .iter()
            .position(|t| t.view.browser().map(|b| b.identifier()) == Some(browser_id))
            .or_else(|| s.tabs.iter().rposition(|t| t.browser_id.is_none()));
        if let Some(i) = i {
            s.tabs[i].browser_id = Some(browser_id);
        }
    });
}

/// `DoClose` do CEF (chamado quando um browser começa a fechar).
/// - Janela fechando (X, Alt+F4, Ctrl+Q, menu) → 0: o CEF segue fechando a janela.
/// - Última aba (Ctrl+W) → marca a janela como fechando e retorna 0: mesmo caminho do X.
/// - Uma aba entre várias → retira a aba da interface e SOLTA as referências à
///   BrowserView (só então o CEF destrói o browser) e retorna 1, para o CEF
///   NÃO fechar a janela. `on_before_close` chega depois e não acha mais a aba.
pub fn on_do_close(browser_id: i32) -> i32 {
    let decision = with(|s| {
        if s.closing {
            return None;
        }
        let p = pos_of(s, browser_id)?;
        if s.tabs.len() <= 1 {
            s.closing = true;
            return None;
        }
        let t = s.tabs.remove(p);
        if s.active >= s.tabs.len() {
            s.active = s.tabs.len() - 1;
        } else if s.active > p {
            s.active -= 1;
        }
        Some((
            t,
            s.active,
            s.ui.as_ref()
                .map(|u| (u.tabstrip.clone(), u.content.clone())),
        ))
    });
    let Some((tab, active, ui)) = decision else {
        return 0;
    };
    if let Some((strip, content)) = ui {
        let mut b: View = (&tab.button).into();
        strip.remove_child_view(Some(&mut b));
        let mut v: View = (&tab.view).into();
        content.remove_child_view(Some(&mut v));
        strip.layout();
    }
    drop(tab);
    log::info!("osmb: aba fechando browser={browser_id} (a janela permanece)");
    activate(active);
    1
}

pub fn on_title(browser_id: i32, title: String) {
    log::info!("osmb: title browser={browser_id} title={title}");
    let Some(i) = tab_index(browser_id) else {
        return;
    };
    let (button, is_active, window) = with(|s| {
        s.tabs[i].title = title.clone();
        (
            s.tabs[i].button.clone(),
            s.active == i,
            s.ui.as_ref().map(|u| u.window.clone()),
        )
    });
    let short: String = title.chars().take(24).collect();
    button.set_text(Some(&cs(if short.is_empty() {
        "Nova aba"
    } else {
        &short
    })));
    if let (true, Some(w)) = (is_active, window) {
        w.set_title(Some(&cs(&format!("{title} — OSMarcio Browser"))));
    }
    if let Some((strip,)) = with(|s| s.ui.as_ref().map(|u| (u.tabstrip.clone(),))) {
        strip.layout();
    }
}

pub fn on_address(browser_id: i32, url: String) {
    let Some(i) = tab_index(browser_id) else {
        return;
    };
    let (is_active, home) = with(|s| {
        let t = &mut s.tabs[i];
        if !url.starts_with("data:") {
            t.home = false;
        }
        t.url = url.clone();
        (s.active == i, t.home)
    });
    if is_active {
        show_address(&url, home);
    }
}

pub fn on_loading(browser_id: i32, loading: bool, can_back: bool, can_fwd: bool) {
    let Some(i) = tab_index(browser_id) else {
        return;
    };
    let Some((back, fwd, reload, active)) = with(|s| {
        s.ui.as_ref().map(|u| {
            (
                u.back.clone(),
                u.forward.clone(),
                u.reload.clone(),
                s.active == i,
            )
        })
    }) else {
        return;
    };
    if active {
        back.set_enabled(can_back as i32);
        fwd.set_enabled(can_fwd as i32);
        reload.set_text(Some(&cs(if loading { "…" } else { "⟳" })));
    }
}

pub fn on_load_end(browser_id: i32, url: &str, status: i32) {
    if !(url.starts_with("http://") || url.starts_with("https://")) || !(200..400).contains(&status)
    {
        return;
    }
    let title = tab_index(browser_id)
        .map(|i| with(|s| s.tabs[i].title.clone()))
        .unwrap_or_default();
    if let Some(f) = EntryFile::in_data_dir("history") {
        let _ = f.append(&Entry {
            url: url.to_string(),
            title,
        });
    }
}

pub fn notify_status(msg: &str) {
    if let Some(l) = with(|s| s.ui.as_ref().map(|u| u.status.clone())) {
        l.set_text(Some(&cs(msg)));
    }
}

/// Caminho de download em ~/Downloads (XDG), sem sobrescrever arquivos.
pub fn download_path(suggested: &str) -> PathBuf {
    let dir = std::env::var_os("XDG_DOWNLOAD_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    let _ = std::fs::create_dir_all(&dir);
    let safe: String = suggested
        .chars()
        .map(|c| if c == '/' || c == '\0' { '_' } else { c })
        .collect();
    let safe = if safe.is_empty() || safe == "." || safe == ".." {
        "download".to_string()
    } else {
        safe
    };
    let mut p = dir.join(&safe);
    let mut n = 1;
    while p.exists() {
        p = dir.join(format!("({n}) {safe}"));
        n += 1;
    }
    p
}
