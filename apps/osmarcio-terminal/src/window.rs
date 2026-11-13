//! Janela principal: um `gtk::Notebook` em que cada página é um `vte::Terminal`.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, pango};
use osmarcio_core::tabs::{CloseShortcut, TabPolicy, normalize_tab_title};
use vte4::prelude::*;

use crate::config::{TerminalConfig, palette};

/// Uma aba: o widget raiz (página do Notebook) e seus controles.
struct Tab {
    root: gtk4::ScrolledWindow,
    term: vte4::Terminal,
    label: gtk4::Label,
    close: gtk4::Button,
    custom_title: RefCell<Option<String>>,
}

struct Inner {
    window: gtk4::ApplicationWindow,
    notebook: gtk4::Notebook,
    tabs: RefCell<Vec<Rc<Tab>>>,
    cfg: TerminalConfig,
    policy: TabPolicy,
}

#[derive(Clone)]
pub struct TermWindow(Rc<Inner>);

impl TermWindow {
    pub fn new(app: &gtk4::Application, cfg: TerminalConfig) -> Self {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("OSMarcio Terminal")
            .default_width(900)
            .default_height(560)
            .icon_name("utilities-terminal")
            .build();

        let notebook = gtk4::Notebook::builder()
            .scrollable(true)
            .show_border(false)
            .build();

        let header = gtk4::HeaderBar::new();
        let new_btn = gtk4::Button::from_icon_name("tab-new-symbolic");
        new_btn.set_tooltip_text(Some("Nova aba (Ctrl+T)"));
        header.pack_start(&new_btn);
        window.set_titlebar(Some(&header));
        window.set_child(Some(&notebook));

        let this = TermWindow(Rc::new(Inner {
            window,
            notebook,
            tabs: RefCell::new(Vec::new()),
            cfg,
            policy: TabPolicy,
        }));

        let w = this.downgrade();
        new_btn.connect_clicked(move |_| {
            if let Some(w) = w.upgrade() {
                w.new_tab();
            }
        });

        // Visibilidade dos botões X acompanha a quantidade de abas.
        for sig in ["page-added", "page-removed"] {
            let w = this.downgrade();
            this.0.notebook.connect_local(sig, false, move |_| {
                if let Some(w) = w.upgrade() {
                    w.refresh_close_controls();
                }
                None
            });
        }
        // Foco no terminal ao trocar de aba.
        this.0.notebook.connect_switch_page(|_, page, _| {
            if let Some(sw) = page.downcast_ref::<gtk4::ScrolledWindow>()
                && let Some(t) = sw.child()
            {
                glib::idle_add_local_once(move || {
                    t.grab_focus();
                });
            }
        });

        // O estado da janela vive enquanto a janela GTK viver: o handler de
        // `destroy` guarda a referência forte; quando a janela é destruída o
        // handler é liberado e o ciclo Rc se desfaz. (Os callbacks usam Weak.)
        let keep = this.clone();
        this.0.window.connect_destroy(move |_| {
            keep.0.tabs.borrow_mut().clear();
        });

        this.install_shortcuts();
        this.new_tab();
        this
    }

    pub fn present(&self) {
        self.0.window.present();
    }

    fn downgrade(&self) -> WeakWin {
        WeakWin(Rc::downgrade(&self.0))
    }

    fn tab_count(&self) -> usize {
        self.0.tabs.borrow().len()
    }

    fn current_tab(&self) -> Option<Rc<Tab>> {
        let page = self.0.notebook.current_page()?;
        let widget = self.0.notebook.nth_page(Some(page))?;
        self.0
            .tabs
            .borrow()
            .iter()
            .find(|t| t.root.upcast_ref::<gtk4::Widget>() == &widget)
            .cloned()
    }

    // ---------------------------------------------------------------- abas

    pub fn new_tab(&self) {
        let cfg = &self.0.cfg;
        let term = vte4::Terminal::new();
        term.set_hexpand(true);
        term.set_vexpand(true);
        term.set_scrollback_lines(cfg.scrollback_lines);
        term.set_font(Some(&pango::FontDescription::from_string(&cfg.font)));
        term.set_cursor_blink_mode(if cfg.cursor_blink {
            vte4::CursorBlinkMode::On
        } else {
            vte4::CursorBlinkMode::Off
        });
        term.set_cursor_shape(match cfg.cursor_shape.as_str() {
            "ibeam" => vte4::CursorShape::Ibeam,
            "underline" => vte4::CursorShape::Underline,
            _ => vte4::CursorShape::Block,
        });
        apply_palette(&term, &cfg.theme);

        let root = gtk4::ScrolledWindow::builder()
            .child(&term)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();

        let label = gtk4::Label::builder()
            .label("Terminal")
            .max_width_chars(24)
            .ellipsize(pango::EllipsizeMode::End)
            .build();
        let close = gtk4::Button::from_icon_name("window-close-symbolic");
        close.add_css_class("flat");
        close.set_tooltip_text(Some("Fechar aba (Ctrl+W)"));
        let tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        tab_box.append(&label);
        tab_box.append(&close);

        let tab = Rc::new(Tab {
            root: root.clone(),
            term: term.clone(),
            label,
            close: close.clone(),
            custom_title: RefCell::new(None),
        });
        self.0.tabs.borrow_mut().push(tab.clone());

        let idx = self.0.notebook.append_page(&root, Some(&tab_box));
        self.0.notebook.set_tab_reorderable(&root, true);

        // Botão X
        let (w, t) = (self.downgrade(), Rc::downgrade(&tab));
        close.connect_clicked(move |_| {
            if let (Some(w), Some(t)) = (w.upgrade(), t.upgrade()) {
                w.close_tab(&t);
            }
        });

        // Menu de contexto da aba: clique direito ou duplo clique.
        let gesture = gtk4::GestureClick::builder().button(0).build();
        let (w, t) = (self.downgrade(), Rc::downgrade(&tab));
        gesture.connect_pressed(move |g, n, _, _| {
            let right = g.current_button() == gdk::BUTTON_SECONDARY;
            if (right || n == 2)
                && let (Some(w), Some(t)) = (w.upgrade(), t.upgrade())
            {
                w.show_tab_menu(&t);
            }
        });
        tab_box.add_controller(gesture);

        // Título vindo do shell (sequência OSC 0/2), salvo se renomeada.
        let t = Rc::downgrade(&tab);
        term.connect_window_title_changed(move |term| {
            if let Some(t) = t.upgrade()
                && t.custom_title.borrow().is_none()
                && let Some(title) = term.window_title()
                && !title.is_empty()
            {
                t.label.set_text(&title);
            }
        });

        // Fim do processo do shell.
        let (w, t) = (self.downgrade(), Rc::downgrade(&tab));
        term.connect_child_exited(move |_, status| {
            log::info!("shell encerrado (status {status})");
            if let (Some(w), Some(t)) = (w.upgrade(), t.upgrade()) {
                if w.tab_count() > 1 {
                    w.remove_tab(&t);
                } else {
                    // `exit` explícito na última aba encerra a janela, como
                    // em qualquer terminal. (Ctrl+W/botão nunca fecham a última.)
                    w.0.window.close();
                }
            }
        });

        self.spawn_shell(&term);
        self.0.notebook.set_current_page(Some(idx));
        term.grab_focus();
    }

    fn spawn_shell(&self, term: &vte4::Terminal) {
        let shell = self.0.cfg.resolve_shell(std::env::var("SHELL").ok());
        let home = glib::home_dir();
        let shell_for_log = shell.clone();
        let argv = [shell.as_str()];
        term.spawn_async(
            vte4::PtyFlags::DEFAULT,
            home.to_str(),
            &argv,
            &[],
            glib::SpawnFlags::DEFAULT,
            || {},
            -1,
            None::<&gio::Cancellable>,
            move |res| match res {
                Ok(pid) => log::info!("shell iniciado: {shell_for_log} (pid {pid:?})"),
                Err(e) => log::error!("falha ao iniciar {shell_for_log}: {e}"),
            },
        );
    }

    /// Fecha uma aba se a política permitir (nunca a última).
    fn close_tab(&self, tab: &Rc<Tab>) {
        if self.0.policy.can_close(self.tab_count()) {
            self.remove_tab(tab);
        }
    }

    fn remove_tab(&self, tab: &Rc<Tab>) {
        if let Some(n) = self.0.notebook.page_num(&tab.root) {
            self.0.tabs.borrow_mut().retain(|t| !Rc::ptr_eq(t, tab));
            // Ao destruir o widget, a VTE envia SIGHUP ao shell da aba.
            self.0.notebook.remove_page(Some(n));
        }
    }

    fn close_others(&self, keep: &Rc<Tab>) {
        let others: Vec<_> = self
            .0
            .tabs
            .borrow()
            .iter()
            .filter(|t| !Rc::ptr_eq(t, keep))
            .cloned()
            .collect();
        for t in others {
            self.remove_tab(&t);
        }
    }

    fn refresh_close_controls(&self) {
        let show = self.0.policy.show_close_buttons(self.tab_count());
        for t in self.0.tabs.borrow().iter() {
            t.close.set_visible(show);
        }
        // Linha estável usada pelo teste de GUI (scripts/test/test-terminal-gui.sh).
        log::info!("abas={} botoes_fechar={show}", self.tab_count());
    }

    fn show_tab_menu(&self, tab: &Rc<Tab>) {
        let pop = gtk4::Popover::new();
        let col = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        col.set_margin_top(6);
        col.set_margin_bottom(6);
        col.set_margin_start(6);
        col.set_margin_end(6);

        let entry = gtk4::Entry::builder()
            .placeholder_text("Nome da aba (vazio = automático)")
            .text(tab.custom_title.borrow().clone().unwrap_or_default())
            .build();
        let rename = gtk4::Button::with_label("Renomear");
        let close_others = gtk4::Button::with_label("Fechar outras abas");
        let close_this = gtk4::Button::with_label("Fechar aba");
        let many = self.tab_count() > 1;
        close_others.set_sensitive(many);
        close_this.set_visible(self.0.policy.can_close(self.tab_count()));
        for b in [&rename, &close_others, &close_this] {
            b.add_css_class("flat");
        }
        col.append(&entry);
        col.append(&rename);
        col.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
        col.append(&close_others);
        col.append(&close_this);
        pop.set_child(Some(&col));
        pop.set_parent(&tab.label);

        let apply = {
            let (t, pop, entry) = (Rc::downgrade(tab), pop.clone(), entry.clone());
            move || {
                if let Some(t) = t.upgrade() {
                    let name = normalize_tab_title(&entry.text());
                    let shown = name
                        .clone()
                        .or_else(|| t.term.window_title().map(|s| s.to_string()))
                        .unwrap_or_else(|| "Terminal".into());
                    t.label.set_text(&shown);
                    *t.custom_title.borrow_mut() = name;
                }
                pop.popdown();
            }
        };
        let a = apply.clone();
        entry.connect_activate(move |_| a());
        rename.connect_clicked(move |_| apply());

        let (w, t, p) = (self.downgrade(), Rc::downgrade(tab), pop.clone());
        close_others.connect_clicked(move |_| {
            p.popdown();
            if let (Some(w), Some(t)) = (w.upgrade(), t.upgrade()) {
                w.close_others(&t);
            }
        });
        let (w, t, p) = (self.downgrade(), Rc::downgrade(tab), pop.clone());
        close_this.connect_clicked(move |_| {
            p.popdown();
            if let (Some(w), Some(t)) = (w.upgrade(), t.upgrade()) {
                w.close_tab(&t);
            }
        });
        pop.connect_closed(|p| {
            let p = p.clone();
            glib::idle_add_local_once(move || p.unparent());
        });
        pop.popup();
        entry.grab_focus();
    }

    // ---------------------------------------------------------- atalhos

    fn install_shortcuts(&self) {
        let keys = gtk4::EventControllerKey::new();
        // Fase de captura: o atalho é visto antes da VTE repassá-lo ao shell.
        keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let w = self.downgrade();
        keys.connect_key_pressed(move |_, key, _, mods| {
            let Some(w) = w.upgrade() else {
                return glib::Propagation::Proceed;
            };
            w.handle_key(key, mods)
        });
        self.0.window.add_controller(keys);
    }

    fn handle_key(&self, key: gdk::Key, mods: gdk::ModifierType) -> glib::Propagation {
        use gdk::ModifierType as M;
        use glib::Propagation::{Proceed, Stop};
        log::debug!("tecla {:?} mods {:?}", key.name(), mods);
        let ctrl = mods.contains(M::CONTROL_MASK);
        let shift = mods.contains(M::SHIFT_MASK);
        if !ctrl {
            return Proceed;
        }
        let plain = self.0.cfg.plain_ctrl_tab_shortcuts;
        let tab_mod_ok = if plain { true } else { shift };
        let lower = key.to_lower();
        let n = self.tab_count();
        let cur = self.0.notebook.current_page().unwrap_or(0) as usize;

        match lower {
            gdk::Key::t if tab_mod_ok => {
                self.new_tab();
                Stop
            }
            gdk::Key::w if tab_mod_ok => match self.0.policy.on_close_shortcut(n) {
                CloseShortcut::CloseTab => {
                    if let Some(t) = self.current_tab() {
                        self.remove_tab(&t);
                    }
                    Stop
                }
                // Última aba: não fecha; Ctrl+W segue para o Bash (apagar palavra).
                CloseShortcut::PassThrough => Proceed,
            },
            gdk::Key::Tab | gdk::Key::Page_Down if !shift => {
                self.0
                    .notebook
                    .set_current_page(Some(self.0.policy.next(n, cur) as u32));
                Stop
            }
            gdk::Key::ISO_Left_Tab | gdk::Key::Tab | gdk::Key::Page_Up => {
                self.0
                    .notebook
                    .set_current_page(Some(self.0.policy.previous(n, cur) as u32));
                Stop
            }
            gdk::Key::c if shift => {
                if let Some(t) = self.current_tab() {
                    t.term.copy_clipboard_format(vte4::Format::Text);
                }
                Stop
            }
            gdk::Key::v if shift => {
                if let Some(t) = self.current_tab() {
                    t.term.paste_clipboard();
                }
                Stop
            }
            gdk::Key::plus | gdk::Key::equal | gdk::Key::KP_Add => self.zoom(1.1),
            gdk::Key::minus | gdk::Key::KP_Subtract => self.zoom(1.0 / 1.1),
            gdk::Key::_0 | gdk::Key::KP_0 => self.zoom(0.0),
            _ => Proceed,
        }
    }

    /// factor 0.0 = restaurar 100%.
    fn zoom(&self, factor: f64) -> glib::Propagation {
        if let Some(t) = self.current_tab() {
            let s = if factor == 0.0 {
                1.0
            } else {
                (t.term.font_scale() * factor).clamp(0.5, 4.0)
            };
            t.term.set_font_scale(s);
        }
        glib::Propagation::Stop
    }
}

struct WeakWin(std::rc::Weak<Inner>);
impl WeakWin {
    fn upgrade(&self) -> Option<TermWindow> {
        self.0.upgrade().map(TermWindow)
    }
}

fn apply_palette(term: &vte4::Terminal, theme: &str) {
    let p = palette(theme);
    let rgba = |s: &str| gdk::RGBA::parse(s).unwrap_or(gdk::RGBA::BLACK);
    let ansi: Vec<gdk::RGBA> = p.ansi.iter().map(|c| rgba(c)).collect();
    let refs: Vec<&gdk::RGBA> = ansi.iter().collect();
    term.set_colors(Some(&rgba(p.fg)), Some(&rgba(p.bg)), &refs);
}
