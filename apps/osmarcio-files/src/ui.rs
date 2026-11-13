//! Janela do OSMarcio Files (GTK4). Regras de arquivos ficam em `fsops`,
//! `listing` e `trash`; aqui só há interface, estado de navegação e o
//! despacho de tarefas longas para uma thread (a janela nunca congela).

use crate::fsops::{self, Progress};
use crate::listing::{self, Entry, SortKey};
use crate::trash;
use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Prefs {
    pub show_hidden: bool,
    pub icon_view: bool,
    pub sort: SortKey,
}
impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            show_hidden: false,
            icon_view: false,
            sort: SortKey::Name,
        }
    }
}
const PREFS_APP: &str = "files";

#[derive(Debug, Clone, PartialEq)]
enum Loc {
    Dir(PathBuf),
    Trash,
}

struct State {
    loc: Loc,
    back: Vec<Loc>,
    fwd: Vec<Loc>,
    entries: Vec<Entry>,
    prefs: Prefs,
    clipboard: Option<(Vec<PathBuf>, bool)>,
    task: Option<gio::Cancellable>,
    search: Option<gio::Cancellable>,
}

struct W {
    window: gtk4::ApplicationWindow,
    store: gio::ListStore,
    selection: gtk4::MultiSelection,
    stack: gtk4::Stack,
    location: gtk4::Entry,
    crumbs: gtk4::Box,
    status: gtk4::Label,
    sidebar: gtk4::ListBox,
    places: RefCell<Vec<Loc>>,
    progress_bar: gtk4::Revealer,
    progress: gtk4::ProgressBar,
    progress_label: gtk4::Label,
    search_bar: gtk4::SearchBar,
    search: gtk4::SearchEntry,
    recursive: gtk4::CheckButton,
    back: gtk4::Button,
    fwd: gtk4::Button,
    up: gtk4::Button,
    menu: gtk4::PopoverMenu,
}

#[derive(Clone)]
pub struct Files(Rc<(W, RefCell<State>)>);

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

impl Files {
    fn w(&self) -> &W {
        &self.0.0
    }
    fn s(&self) -> std::cell::RefMut<'_, State> {
        self.0.1.borrow_mut()
    }

    pub fn new(app: &gtk4::Application, start: PathBuf) -> Self {
        let prefs: Prefs = osmarcio_core::config::load_or_default(PREFS_APP);
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("OSMarcio Files")
            .icon_name("osmarcio-files")
            .default_width(1000)
            .default_height(640)
            .build();

        // Cabeçalho -------------------------------------------------------
        let header = gtk4::HeaderBar::new();
        let btn = |icon: &str, tip: &str, action: &str| {
            let b = gtk4::Button::from_icon_name(icon);
            b.set_tooltip_text(Some(tip));
            b.set_action_name(Some(action));
            b
        };
        let back = btn("go-previous-symbolic", "Voltar (Alt+←)", "win.back");
        let fwd = btn("go-next-symbolic", "Avançar (Alt+→)", "win.forward");
        let up = btn("go-up-symbolic", "Pasta acima (Alt+↑)", "win.up");
        header.pack_start(&back);
        header.pack_start(&fwd);
        header.pack_start(&up);
        header.pack_start(&btn(
            "go-home-symbolic",
            "Pasta pessoal (Alt+Home)",
            "win.home",
        ));
        header.pack_start(&btn(
            "view-refresh-symbolic",
            "Atualizar (F5)",
            "win.refresh",
        ));
        let location = gtk4::Entry::builder()
            .hexpand(true)
            .placeholder_text("Caminho (Ctrl+L)")
            .build();
        location.set_width_chars(40);
        header.set_title_widget(Some(&location));

        let main_menu = gio::Menu::new();
        let s1 = gio::Menu::new();
        s1.append(Some("Nova pasta"), Some("win.new-folder"));
        s1.append(Some("Novo arquivo vazio"), Some("win.new-file"));
        main_menu.append_section(None, &s1);
        let s2 = gio::Menu::new();
        s2.append(Some("Mostrar arquivos ocultos"), Some("win.show-hidden"));
        s2.append(Some("Ver como ícones"), Some("win.icon-view"));
        let sort = gio::Menu::new();
        for (l, k) in [
            ("Nome", "name"),
            ("Tipo", "type"),
            ("Tamanho", "size"),
            ("Data de modificação", "modified"),
        ] {
            sort.append(Some(l), Some(&format!("win.sort::{k}")));
        }
        s2.append_submenu(Some("Ordenar por"), &sort);
        main_menu.append_section(None, &s2);
        let s3 = gio::Menu::new();
        s3.append(Some("Adicionar pasta aos favoritos"), Some("win.bookmark"));
        s3.append(Some("Esvaziar lixeira…"), Some("win.empty-trash"));
        s3.append(Some("Sobre o OSMarcio Files"), Some("win.about"));
        main_menu.append_section(None, &s3);
        header.pack_end(
            &gtk4::MenuButton::builder()
                .icon_name("open-menu-symbolic")
                .menu_model(&main_menu)
                .tooltip_text("Menu")
                .build(),
        );
        header.pack_end(&btn(
            "system-search-symbolic",
            "Buscar (Ctrl+F)",
            "win.search",
        ));
        window.set_titlebar(Some(&header));

        // Barra lateral -----------------------------------------------------
        let sidebar = gtk4::ListBox::new();
        sidebar.add_css_class("navigation-sidebar");
        let side_scroll = gtk4::ScrolledWindow::builder()
            .child(&sidebar)
            .width_request(190)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();

        // Visões: lista detalhada e ícones, sobre o mesmo modelo/seleção ----
        let store = gio::ListStore::new::<glib::BoxedAnyObject>();
        let selection = gtk4::MultiSelection::new(Some(store.clone()));
        let list = gtk4::ColumnView::new(Some(selection.clone()));
        list.set_show_column_separators(true);
        let grid = gtk4::GridView::new(Some(selection.clone()), Some(grid_factory()));
        grid.set_max_columns(12);
        let stack = gtk4::Stack::new();
        stack.add_named(
            &gtk4::ScrolledWindow::builder()
                .child(&list)
                .vexpand(true)
                .build(),
            Some("list"),
        );
        stack.add_named(
            &gtk4::ScrolledWindow::builder()
                .child(&grid)
                .vexpand(true)
                .build(),
            Some("icons"),
        );

        let crumbs = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        crumbs.set_margin_start(6);
        let search = gtk4::SearchEntry::builder()
            .placeholder_text("Buscar nesta pasta pelo nome")
            .hexpand(true)
            .build();
        let recursive = gtk4::CheckButton::with_label("Incluir subpastas");
        let sb = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        sb.append(&search);
        sb.append(&recursive);
        let search_bar = gtk4::SearchBar::builder()
            .child(&sb)
            .show_close_button(true)
            .build();
        search_bar.connect_entry(&search);

        let progress = gtk4::ProgressBar::builder()
            .hexpand(true)
            .show_text(true)
            .build();
        let progress_label = gtk4::Label::new(None);
        let cancel = gtk4::Button::with_label("Cancelar");
        cancel.set_action_name(Some("win.cancel-task"));
        let pb = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        pb.set_margin_start(8);
        pb.set_margin_end(8);
        pb.append(&progress_label);
        pb.append(&progress);
        pb.append(&cancel);
        let progress_bar = gtk4::Revealer::builder().child(&pb).build();
        let status = gtk4::Label::builder()
            .xalign(0.0)
            .margin_start(8)
            .margin_end(8)
            .margin_top(2)
            .margin_bottom(2)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();

        let right = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        right.append(&search_bar);
        right.append(&crumbs);
        right.append(&stack);
        right.append(&progress_bar);
        right.append(&status);
        let paned = gtk4::Paned::builder()
            .start_child(&side_scroll)
            .end_child(&right)
            .position(190)
            .shrink_start_child(false)
            .build();
        window.set_child(Some(&paned));

        // Menu de contexto (mesmas ações da janela).
        let ctx = gio::Menu::new();
        let c1 = gio::Menu::new();
        c1.append(Some("Abrir"), Some("win.open"));
        ctx.append_section(None, &c1);
        let c2 = gio::Menu::new();
        c2.append(Some("Recortar"), Some("win.cut"));
        c2.append(Some("Copiar"), Some("win.copy"));
        c2.append(Some("Colar"), Some("win.paste"));
        c2.append(Some("Renomear…"), Some("win.rename"));
        ctx.append_section(None, &c2);
        let c3 = gio::Menu::new();
        c3.append(Some("Mover para a lixeira"), Some("win.trash"));
        c3.append(Some("Restaurar da lixeira"), Some("win.restore"));
        c3.append(Some("Excluir permanentemente…"), Some("win.delete"));
        ctx.append_section(None, &c3);
        let c4 = gio::Menu::new();
        c4.append(Some("Nova pasta"), Some("win.new-folder"));
        c4.append(Some("Novo arquivo vazio"), Some("win.new-file"));
        c4.append(Some("Propriedades"), Some("win.properties"));
        ctx.append_section(None, &c4);
        let menu = gtk4::PopoverMenu::from_model(Some(&ctx));
        menu.set_parent(&stack);
        menu.set_has_arrow(false);

        let state = State {
            loc: Loc::Dir(start),
            back: Vec::new(),
            fwd: Vec::new(),
            entries: Vec::new(),
            prefs,
            clipboard: None,
            task: None,
            search: None,
        };
        let w = W {
            window,
            store,
            selection,
            stack,
            location,
            crumbs,
            status,
            sidebar,
            places: RefCell::new(Vec::new()),
            progress_bar,
            progress,
            progress_label,
            search_bar,
            search,
            recursive,
            back,
            fwd,
            up,
            menu,
        };
        let this = Files(Rc::new((w, RefCell::new(state))));
        this.setup_columns(&list);
        this.setup_actions(app);
        this.setup_signals(&list, &grid);
        this.fill_sidebar();
        let icon_view = this.s().prefs.icon_view;
        this.w()
            .stack
            .set_visible_child_name(if icon_view { "icons" } else { "list" });
        this.reload();
        this.w().window.present();
        log::info!("osmf: janela criada");
        this
    }

    // ------------------------------------------------------------------ colunas
    fn setup_columns(&self, list: &gtk4::ColumnView) {
        let name_f = gtk4::SignalListItemFactory::new();
        name_f.connect_setup(|_, li| {
            let li = li.downcast_ref::<gtk4::ListItem>().unwrap();
            let b = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            b.append(&gtk4::Image::new());
            b.append(
                &gtk4::Label::builder()
                    .xalign(0.0)
                    .width_chars(18)
                    .ellipsize(gtk4::pango::EllipsizeMode::Middle)
                    .build(),
            );
            li.set_child(Some(&b));
        });
        name_f.connect_bind(|_, li| {
            let li = li.downcast_ref::<gtk4::ListItem>().unwrap();
            let (Some(obj), Some(b)) = (li.item(), li.child()) else {
                return;
            };
            let e = obj
                .downcast_ref::<glib::BoxedAnyObject>()
                .unwrap()
                .borrow::<Entry>()
                .clone();
            let img = b.first_child().and_downcast::<gtk4::Image>().unwrap();
            match &e.icon {
                Some(i) => img.set_from_gicon(i),
                None => img.set_icon_name(Some("text-x-generic")),
            }
            let l = b.last_child().and_downcast::<gtk4::Label>().unwrap();
            l.set_text(&if e.is_symlink {
                format!("{} ↗", e.name)
            } else {
                e.name.clone()
            });
            b.set_tooltip_text(Some(&e.path.display().to_string()));
            b.set_widget_name(&format!("osmf-pos-{}", li.position()));
        });
        let col = gtk4::ColumnViewColumn::new(Some("Nome"), Some(name_f));
        col.set_expand(true);
        list.append_column(&col);
        type Fmt = fn(&Entry) -> String;
        let cols: [(&str, Fmt); 4] = [
            ("Tamanho", |e| {
                if e.is_dir {
                    "—".into()
                } else {
                    listing::human_size(e.size)
                }
            }),
            ("Tipo", |e| e.description.clone()),
            ("Modificado", |e| match &e.trash {
                Some(t) => format!("excluído {}", t.deleted),
                None => listing::date(e.modified),
            }),
            ("Permissões", |e| match &e.trash {
                Some(t) => format!(
                    "de {}",
                    t.original
                        .parent()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                ),
                None => listing::perms(e.mode),
            }),
        ];
        for (title, f) in cols {
            let fac = gtk4::SignalListItemFactory::new();
            fac.connect_setup(|_, li| {
                let li = li.downcast_ref::<gtk4::ListItem>().unwrap();
                // Colunas secundárias nunca espremem o nome (caminhos longos ganham "…").
                li.set_child(Some(
                    &gtk4::Label::builder()
                        .xalign(0.0)
                        .max_width_chars(28)
                        .ellipsize(gtk4::pango::EllipsizeMode::Middle)
                        .build(),
                ));
            });
            fac.connect_bind(move |_, li| {
                let li = li.downcast_ref::<gtk4::ListItem>().unwrap();
                let (Some(obj), Some(l)) = (li.item(), li.child().and_downcast::<gtk4::Label>())
                else {
                    return;
                };
                l.set_text(&f(&obj
                    .downcast_ref::<glib::BoxedAnyObject>()
                    .unwrap()
                    .borrow::<Entry>()));
                l.set_widget_name(&format!("osmf-pos-{}", li.position()));
            });
            list.append_column(&gtk4::ColumnViewColumn::new(Some(title), Some(fac)));
        }
    }

    // ------------------------------------------------------------------ ações
    fn setup_actions(&self, app: &gtk4::Application) {
        let win = &self.w().window;
        let add = |name: &str, accels: &[&str], f: fn(&Files)| {
            let a = gio::SimpleAction::new(name, None);
            let me = self.clone();
            a.connect_activate(move |_, _| f(&me));
            win.add_action(&a);
            if !accels.is_empty() {
                app.set_accels_for_action(&format!("win.{name}"), accels);
            }
        };
        add("back", &["<Alt>Left"], |f| f.history(true));
        add("forward", &["<Alt>Right"], |f| f.history(false));
        add("up", &["<Alt>Up"], |f| f.go_up());
        add("home", &["<Alt>Home"], |f| f.navigate(Loc::Dir(home())));
        add("refresh", &["F5", "<Control>r"], |f| f.reload());
        add("location", &["<Control>l"], |f| {
            f.w().location.grab_focus();
            f.w().location.select_region(0, -1);
        });
        add("search", &["<Control>f"], |f| {
            f.w().search_bar.set_search_mode(true);
            f.w().search.grab_focus();
        });
        add("open", &[], |f| {
            if let Some(e) = f.selected().into_iter().next() {
                f.open(&e);
            }
        });
        add("copy", &["<Control>c"], |f| f.to_clipboard(false));
        add("cut", &["<Control>x"], |f| f.to_clipboard(true));
        add("paste", &["<Control>v"], |f| f.paste());
        add("rename", &["F2"], |f| f.rename());
        add("trash", &["Delete"], |f| f.trash());
        add("delete", &["<Shift>Delete"], |f| f.delete_permanently());
        add("restore", &["<Control><Shift>r"], |f| f.restore());
        add("empty-trash", &[], |f| f.empty_trash());
        add("properties", &["<Alt>Return"], |f| {
            if let Some(e) = f.selected().into_iter().next() {
                crate::dialogs::properties(&f.w().window, &e);
            }
        });
        add("new-folder", &["<Control><Shift>n"], |f| f.new_item(true));
        add("new-file", &[], |f| f.new_item(false));
        add("select-all", &["<Control>a"], |f| {
            f.w().selection.select_all();
        });
        add("cancel-task", &["Escape"], |f| {
            if let Some(c) = f.s().task.as_ref() {
                c.cancel();
                log::info!("osmf: cancelamento solicitado");
            } else if f.w().search_bar.is_search_mode() {
                f.w().search_bar.set_search_mode(false);
            }
        });
        add("bookmark", &[], |f| f.add_bookmark());
        add("about", &["F1"], |f| crate::dialogs::about(&f.w().window));
        add("show-hidden", &["<Control>h"], |f| {
            let v = {
                let mut s = f.s();
                s.prefs.show_hidden = !s.prefs.show_hidden;
                s.prefs.show_hidden
            };
            log::info!("osmf: ocultos={v}");
            f.save_prefs();
            f.reload();
        });
        add("icon-view", &["<Control>1"], |f| f.set_icon_view(true));
        add("list-view", &["<Control>2"], |f| f.set_icon_view(false));
        let sort = gio::SimpleAction::new("sort", Some(glib::VariantTy::STRING));
        let me = self.clone();
        sort.connect_activate(move |_, v| {
            let k = match v.and_then(|v| v.get::<String>()).as_deref() {
                Some("type") => SortKey::Type,
                Some("size") => SortKey::Size,
                Some("modified") => SortKey::Modified,
                _ => SortKey::Name,
            };
            me.s().prefs.sort = k;
            me.save_prefs();
            me.reload();
        });
        win.add_action(&sort);
    }

    fn setup_signals(&self, list: &gtk4::ColumnView, grid: &gtk4::GridView) {
        let me = self.clone();
        list.connect_activate(move |_, pos| me.activate(pos));
        let me = self.clone();
        grid.connect_activate(move |_, pos| me.activate(pos));
        let me = self.clone();
        self.w().location.connect_activate(move |e| {
            let txt = e.text().to_string();
            let p = if let Some(rest) = txt.strip_prefix('~') {
                home().join(rest.trim_start_matches('/'))
            } else {
                PathBuf::from(&txt)
            };
            if txt.trim() == "lixeira:" || txt.trim() == "trash:" {
                me.navigate(Loc::Trash);
            } else {
                me.navigate(Loc::Dir(p));
            }
            // Devolve o foco à lista: os atalhos (Ctrl+C/V, Delete, F2…) voltam
            // a agir sobre os arquivos, e não sobre o texto da barra.
            me.focus_view();
        });
        let me = self.clone();
        self.w().sidebar.connect_row_activated(move |_, row| {
            let loc = me.w().places.borrow().get(row.index() as usize).cloned();
            if let Some(l) = loc {
                me.navigate(l);
            }
        });
        let me = self.clone();
        self.w()
            .search
            .connect_search_changed(move |_| me.run_search());
        let me = self.clone();
        self.w().recursive.connect_toggled(move |_| me.run_search());
        let me = self.clone();
        self.w()
            .search_bar
            .connect_search_mode_enabled_notify(move |b| {
                if !b.is_search_mode() {
                    me.cancel_search();
                    me.reload();
                }
            });
        // Clique direito: seleciona o item sob o cursor (se não estava) e abre o menu.
        let g = gtk4::GestureClick::builder()
            .button(3)
            .propagation_phase(gtk4::PropagationPhase::Capture)
            .build();
        let me = self.clone();
        g.connect_pressed(move |g, _, x, y| {
            let stack = &me.w().stack;
            let mut cur = stack.pick(x, y, gtk4::PickFlags::DEFAULT);
            while let Some(wd) = cur {
                if let Some(pos) = wd
                    .widget_name()
                    .strip_prefix("osmf-pos-")
                    .and_then(|p| p.parse::<u32>().ok())
                {
                    if !me.w().selection.is_selected(pos) {
                        me.w().selection.select_item(pos, true);
                    }
                    break;
                }
                cur = wd.parent();
            }
            me.w()
                .menu
                .set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            me.w().menu.popup();
            g.set_state(gtk4::EventSequenceState::Claimed);
        });
        self.w().stack.add_controller(g);
        // Backspace = pasta acima (quando o foco não está num campo de texto).
        let keys = gtk4::EventControllerKey::new();
        let me = self.clone();
        keys.connect_key_pressed(move |_, k, _, _| {
            let typing = gtk4::prelude::GtkWindowExt::focus(&me.w().window)
                .is_some_and(|f| f.is::<gtk4::Text>());
            if k == gdk::Key::BackSpace && !typing {
                me.go_up();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        self.w().window.add_controller(keys);
    }

    fn fill_sidebar(&self) {
        let sb = &self.w().sidebar;
        while let Some(c) = sb.first_child() {
            sb.remove(&c);
        }
        let mut places = Vec::new();
        let mut row = |icon: &str, label: &str, loc: Loc| {
            let b = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            b.append(&gtk4::Image::from_icon_name(icon));
            b.append(
                &gtk4::Label::builder()
                    .label(label)
                    .xalign(0.0)
                    .ellipsize(gtk4::pango::EllipsizeMode::End)
                    .build(),
            );
            sb.append(&b);
            places.push(loc);
        };
        let h = home();
        row("user-home", "Pasta pessoal", Loc::Dir(h.clone()));
        use glib::UserDirectory as U;
        for (dir, icon) in [
            (U::Desktop, "user-desktop"),
            (U::Documents, "folder-documents"),
            (U::Downloads, "folder-download"),
            (U::Pictures, "folder-pictures"),
            (U::Music, "folder-music"),
            (U::Videos, "folder-videos"),
        ] {
            // Só pastas que existem de verdade (nada é criado para encher o menu).
            if let Some(p) = glib::user_special_dir(dir).filter(|p| p != &h && p.is_dir()) {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                row(icon, &name, Loc::Dir(p));
            }
        }
        row("user-trash", "Lixeira", Loc::Trash);
        // Favoritos (mesmo arquivo do GTK: ~/.config/gtk-3.0/bookmarks).
        for (p, name) in bookmarks() {
            if p.is_dir() {
                row("user-bookmarks", &name, Loc::Dir(p));
            }
        }
        row(
            "drive-harddisk",
            "Sistema de arquivos",
            Loc::Dir(PathBuf::from("/")),
        );
        for m in gio::VolumeMonitor::get().mounts() {
            if let Some(p) = m.root().path().filter(|p| p != Path::new("/")) {
                row("drive-removable-media", &m.name(), Loc::Dir(p));
            }
        }
        *self.w().places.borrow_mut() = places;
    }

    // ------------------------------------------------------------------ navegação
    fn navigate(&self, loc: Loc) {
        let cur = self.s().loc.clone();
        if cur == loc {
            self.reload();
            return;
        }
        if let Loc::Dir(p) = &loc {
            // Testa antes de trocar: erro não perde a pasta atual.
            if let Err(e) = listing::list_dir(p, false) {
                self.error(&format!("Não foi possível abrir {}: {e}", p.display()));
                return;
            }
        }
        {
            let mut s = self.s();
            s.back.push(cur);
            s.fwd.clear();
            s.loc = loc;
        }
        self.w().search_bar.set_search_mode(false);
        self.reload();
    }

    fn history(&self, back: bool) {
        let next = {
            let mut s = self.s();
            let n = if back { s.back.pop() } else { s.fwd.pop() };
            if let Some(n) = &n {
                let cur = std::mem::replace(&mut s.loc, n.clone());
                if back {
                    s.fwd.push(cur)
                } else {
                    s.back.push(cur)
                }
            }
            n
        };
        if next.is_some() {
            self.reload();
        }
    }

    fn go_up(&self) {
        let parent = match &self.s().loc {
            Loc::Dir(p) => p.parent().map(Path::to_path_buf),
            Loc::Trash => Some(home()),
        };
        if let Some(p) = parent {
            self.navigate(Loc::Dir(p));
        }
    }

    fn current_dir(&self) -> Option<PathBuf> {
        match &self.s().loc {
            Loc::Dir(p) => Some(p.clone()),
            Loc::Trash => None,
        }
    }

    pub fn reload(&self) {
        let (loc, hidden, sort) = {
            let s = self.s();
            (s.loc.clone(), s.prefs.show_hidden, s.prefs.sort)
        };
        let result = match &loc {
            Loc::Dir(p) => listing::list_dir(p, hidden),
            Loc::Trash => Ok(trash::list()
                .into_iter()
                .filter_map(|t| {
                    let mut e = listing::stat(&t.path).ok()?;
                    e.name = t
                        .original
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or(e.name);
                    e.trash = Some(t);
                    Some(e)
                })
                .collect()),
        };
        let mut entries = match result {
            Ok(v) => v,
            Err(e) => {
                self.error(&format!("Não foi possível ler a pasta: {e}"));
                Vec::new()
            }
        };
        if loc != Loc::Trash {
            listing::sort(&mut entries, sort);
        }
        self.show_entries(entries);
        let (title, path_txt) = match &loc {
            Loc::Dir(p) => (
                p.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "/".into()),
                p.display().to_string(),
            ),
            Loc::Trash => ("Lixeira".into(), "lixeira:".into()),
        };
        self.w()
            .window
            .set_title(Some(&format!("{title} — OSMarcio Files")));
        self.w().location.set_text(&path_txt);
        self.update_crumbs(&loc);
        let (has_back, has_fwd) = {
            let s = self.s();
            (!s.back.is_empty(), !s.fwd.is_empty())
        };
        self.w().back.set_sensitive(has_back);
        self.w().fwd.set_sensitive(has_fwd);
        self.w()
            .up
            .set_sensitive(!matches!(&loc, Loc::Dir(p) if p == Path::new("/")));
        log::info!("osmf: pasta {path_txt} itens={}", self.s().entries.len());
    }

    fn show_entries(&self, entries: Vec<Entry>) {
        let n = entries.len();
        let dirs = entries.iter().filter(|e| e.is_dir).count();
        let items: Vec<glib::BoxedAnyObject> = entries
            .iter()
            .cloned()
            .map(glib::BoxedAnyObject::new)
            .collect();
        self.w().store.splice(0, self.w().store.n_items(), &items);
        self.s().entries = entries;
        let hidden = if self.s().prefs.show_hidden {
            " · ocultos visíveis"
        } else {
            ""
        };
        self.w()
            .status
            .set_text(&format!("{n} itens ({dirs} pastas){hidden}"));
        self.w().status.remove_css_class("error");
    }

    fn update_crumbs(&self, loc: &Loc) {
        let c = &self.w().crumbs;
        while let Some(ch) = c.first_child() {
            c.remove(&ch);
        }
        let Loc::Dir(p) = loc else {
            // Botão plano, como nas pastas: a trilha mantém a altura e a lista não "pula".
            let b = gtk4::Button::with_label("Lixeira");
            b.add_css_class("flat");
            c.append(&b);
            return;
        };
        let mut acc = PathBuf::new();
        for comp in p.components() {
            acc.push(comp);
            let label = match comp {
                std::path::Component::RootDir => "/".to_string(),
                c => c.as_os_str().to_string_lossy().into_owned(),
            };
            let b = gtk4::Button::with_label(&label);
            b.add_css_class("flat");
            let (me, target) = (self.clone(), acc.clone());
            b.connect_clicked(move |_| me.navigate(Loc::Dir(target.clone())));
            c.append(&b);
        }
    }

    fn focus_view(&self) {
        if let Some(view) = self
            .w()
            .stack
            .visible_child()
            .and_downcast::<gtk4::ScrolledWindow>()
            .and_then(|sw| sw.child())
        {
            view.grab_focus();
        }
    }

    fn activate(&self, pos: u32) {
        let e = self.s().entries.get(pos as usize).cloned();
        if let Some(e) = e {
            self.open(&e);
        }
    }

    fn open(&self, e: &Entry) {
        if e.trash.is_some() {
            self.info("Itens da lixeira precisam ser restaurados antes de abrir.");
        } else if e.is_dir {
            self.navigate(Loc::Dir(e.path.clone()));
        } else if listing::is_executable_type(&e.content_type) {
            // Segurança: o gerenciador de arquivos não executa programas/scripts.
            self.info(&format!("\"{}\" é um programa ou script ({}). Por segurança o OSMarcio Files não o executa; use o terminal se for intencional.", e.name, e.content_type));
            log::warn!("osmf: execução recusada {}", e.path.display());
        } else {
            let uri = gio::File::for_path(&e.path).uri();
            match gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>) {
                Ok(()) => log::info!("osmf: aberto {} ({})", e.path.display(), e.content_type),
                Err(err) => self.error(&format!(
                    "Nenhum aplicativo abre \"{}\" ({}): {}",
                    e.name,
                    e.description,
                    err.message()
                )),
            }
        }
    }

    fn selected(&self) -> Vec<Entry> {
        let sel = self.w().selection.selection();
        let s = self.s();
        (0..sel.size())
            .filter_map(|i| s.entries.get(sel.nth(i as u32) as usize).cloned())
            .collect()
    }

    // ------------------------------------------------------------------ operações
    fn to_clipboard(&self, cut: bool) {
        let paths: Vec<PathBuf> = self
            .selected()
            .into_iter()
            .filter(|e| e.trash.is_none())
            .map(|e| e.path)
            .collect();
        if paths.is_empty() {
            return;
        }
        // Também como texto (caminhos) na área de transferência do sistema.
        let txt = paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        self.w().window.clipboard().set_text(&txt);
        self.w().status.set_text(&format!(
            "{} item(ns) {} — cole com Ctrl+V",
            paths.len(),
            if cut { "recortado(s)" } else { "copiado(s)" }
        ));
        log::info!(
            "osmf: {} {} itens",
            if cut { "recortar" } else { "copiar" },
            paths.len()
        );
        self.s().clipboard = Some((paths, cut));
    }

    fn paste(&self) {
        let Some(dest) = self.current_dir() else {
            return;
        };
        let Some((paths, cut)) = self.s().clipboard.clone() else {
            return;
        };
        if cut {
            self.s().clipboard = None;
        }
        let what = if cut { "Movendo" } else { "Copiando" };
        self.run_task(
            format!("{what} {} item(ns)…", paths.len()),
            move |c, cb| {
                let r = if cut {
                    fsops::move_items(&paths, &dest, c, cb)
                } else {
                    fsops::copy_items(&paths, &dest, c, cb)
                };
                r.map(|v| {
                    format!(
                        "{} item(ns) {}",
                        v.len(),
                        if cut { "movido(s)" } else { "copiado(s)" }
                    )
                })
            },
        );
    }

    fn trash(&self) {
        let sel = self.selected();
        if sel.iter().any(|e| e.trash.is_some()) {
            return self.delete_permanently();
        }
        let paths: Vec<PathBuf> = sel.into_iter().map(|e| e.path).collect();
        if paths.is_empty() {
            return;
        }
        match fsops::trash(&paths) {
            Ok(()) => {
                log::info!("osmf: lixeira {} itens", paths.len());
                self.reload();
                self.w().status.set_text(&format!(
                    "{} item(ns) movido(s) para a lixeira",
                    paths.len()
                ));
            }
            Err(e) => self.error(&format!("Não foi possível mover para a lixeira: {e}")),
        }
    }

    fn delete_permanently(&self) {
        let sel = self.selected();
        if sel.is_empty() {
            return;
        }
        let names: Vec<String> = sel.iter().take(5).map(|e| e.name.clone()).collect();
        let me = self.clone();
        crate::dialogs::confirm(
            &self.w().window,
            &format!("Excluir permanentemente {} item(ns)?", sel.len()),
            &format!(
                "{}{}\n\nEsta ação NÃO pode ser desfeita: os itens não vão para a lixeira.",
                names.join("\n"),
                if sel.len() > 5 { "\n…" } else { "" }
            ),
            "Excluir permanentemente",
            move || {
                let in_trash: Vec<trash::Trashed> =
                    sel.iter().filter_map(|e| e.trash.clone()).collect();
                let paths: Vec<PathBuf> = sel
                    .iter()
                    .filter(|e| e.trash.is_none())
                    .map(|e| e.path.clone())
                    .collect();
                me.run_task("Excluindo…".into(), move |_, _| {
                    trash::purge(&in_trash)?;
                    fsops::delete_permanently(&paths)?;
                    Ok(format!(
                        "{} item(ns) excluído(s) permanentemente",
                        in_trash.len() + paths.len()
                    ))
                });
            },
        );
    }

    fn restore(&self) {
        let items: Vec<trash::Trashed> = self
            .selected()
            .into_iter()
            .filter_map(|e| e.trash)
            .collect();
        if items.is_empty() {
            return;
        }
        let mut errs = Vec::new();
        for t in &items {
            match trash::restore(t) {
                Ok(p) => log::info!("osmf: restaurado {}", p.display()),
                Err(e) => errs.push(e),
            }
        }
        self.reload();
        if errs.is_empty() {
            self.w()
                .status
                .set_text(&format!("{} item(ns) restaurado(s)", items.len()));
        } else {
            self.error(&errs.join("\n"));
        }
    }

    fn empty_trash(&self) {
        let items = trash::list();
        if items.is_empty() {
            return self.info("A lixeira está vazia.");
        }
        let me = self.clone();
        crate::dialogs::confirm(
            &self.w().window,
            &format!("Esvaziar a lixeira ({} itens)?", items.len()),
            "Os itens serão excluídos permanentemente. Esta ação não pode ser desfeita.",
            "Esvaziar lixeira",
            move || {
                let items = items.clone();
                me.run_task("Esvaziando a lixeira…".into(), move |_, _| {
                    trash::purge(&items).map(|_| "Lixeira esvaziada".into())
                });
            },
        );
    }

    fn rename(&self) {
        let Some(e) = self
            .selected()
            .into_iter()
            .next()
            .filter(|e| e.trash.is_none())
        else {
            return;
        };
        let me = self.clone();
        crate::dialogs::ask_name(&self.w().window, "Renomear", &e.name, move |name| {
            match fsops::rename(&e.path, &name) {
                Ok(p) => {
                    log::info!("osmf: renomeado {} -> {}", e.path.display(), p.display());
                    me.reload();
                    Ok(())
                }
                Err(err) => Err(err),
            }
        });
    }

    fn new_item(&self, folder: bool) {
        let Some(dir) = self.current_dir() else {
            return;
        };
        let me = self.clone();
        let (title, initial) = if folder {
            ("Nova pasta", "Nova pasta")
        } else {
            ("Novo arquivo vazio", "Novo arquivo.txt")
        };
        crate::dialogs::ask_name(&self.w().window, title, initial, move |name| {
            let r = if folder {
                fsops::create_folder(&dir, &name)
            } else {
                fsops::create_file(&dir, &name)
            };
            r.map(|p| {
                log::info!("osmf: criado {}", p.display());
                me.reload();
            })
        });
    }

    fn add_bookmark(&self) {
        let Some(dir) = self.current_dir() else {
            return;
        };
        let file = osmarcio_core::config::config_dir()
            .map(|d| d.parent().unwrap_or(&d).join("gtk-3.0/bookmarks"));
        let Ok(file) = file else { return };
        let uri = gio::File::for_path(&dir).uri();
        let cur = std::fs::read_to_string(&file).unwrap_or_default();
        if cur
            .lines()
            .any(|l| l.split(' ').next() == Some(uri.as_str()))
        {
            return;
        }
        let _ = std::fs::create_dir_all(file.parent().unwrap());
        if std::fs::write(&file, format!("{cur}{uri}\n")).is_ok() {
            self.fill_sidebar();
        }
    }

    fn set_icon_view(&self, icons: bool) {
        self.s().prefs.icon_view = icons;
        self.save_prefs();
        self.w()
            .stack
            .set_visible_child_name(if icons { "icons" } else { "list" });
    }

    fn save_prefs(&self) {
        let p = self.s().prefs.clone();
        if let Ok(path) = osmarcio_core::config::config_path(PREFS_APP) {
            let _ = osmarcio_core::config::save_to(&path, &p);
        }
    }

    // ------------------------------------------------------------------ busca
    fn cancel_search(&self) {
        if let Some(c) = self.s().search.take() {
            c.cancel();
        }
    }

    fn run_search(&self) {
        self.cancel_search();
        let q = self.w().search.text().to_string();
        let Some(dir) = self.current_dir() else {
            return;
        };
        if q.trim().is_empty() {
            return self.reload();
        }
        let recursive = self.w().recursive.is_active();
        let hidden = self.s().prefs.show_hidden;
        let c = gio::Cancellable::new();
        self.s().search = Some(c.clone());
        self.w().status.set_text("Buscando…");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(fsops::search(&dir, &q, recursive, hidden, &c, 5000));
        });
        let me = self.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
            match rx.try_recv() {
                Ok(paths) => {
                    let mut v: Vec<Entry> =
                        paths.iter().filter_map(|p| listing::stat(p).ok()).collect();
                    listing::sort(&mut v, me.s().prefs.sort);
                    let n = v.len();
                    me.show_entries(v);
                    me.w().status.set_text(&format!(
                        "{n} resultado(s){}",
                        if n >= 5000 { " (limite)" } else { "" }
                    ));
                    log::info!("osmf: busca resultados={n}");
                    glib::ControlFlow::Break
                }
                Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(_) => glib::ControlFlow::Break,
            }
        });
    }

    // ------------------------------------------------------------------ tarefas
    /// Executa uma operação numa thread, com barra de progresso e cancelamento.
    /// O sucesso só é anunciado quando a função retorna Ok.
    fn run_task<F>(&self, label: String, f: F)
    where
        F: FnOnce(&gio::Cancellable, &mut dyn FnMut(&Progress)) -> Result<String, String>
            + Send
            + 'static,
    {
        if self.s().task.is_some() {
            return self.info("Aguarde a operação em andamento terminar (ou cancele-a).");
        }
        let c = gio::Cancellable::new();
        self.s().task = Some(c.clone());
        let w = self.w();
        w.progress_label.set_text(&label);
        w.progress.set_fraction(0.0);
        w.progress.set_text(Some(""));
        w.progress_bar.set_reveal_child(true);
        enum Msg {
            P(Progress),
            Done(Result<String, String>),
        }
        let (tx, rx) = mpsc::channel::<Msg>();
        std::thread::spawn(move || {
            let txp = tx.clone();
            let mut last = std::time::Instant::now();
            let mut cb = |p: &Progress| {
                if last.elapsed().as_millis() > 100 {
                    last = std::time::Instant::now();
                    let _ = txp.send(Msg::P(p.clone()));
                }
            };
            let r = f(&c, &mut cb);
            let _ = tx.send(Msg::Done(r));
        });
        let me = self.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            while let Ok(m) = rx.try_recv() {
                match m {
                    Msg::P(p) => {
                        let frac = if p.total > 0 {
                            p.done as f64 / p.total as f64
                        } else {
                            0.0
                        };
                        me.w().progress.set_fraction(frac.min(1.0));
                        me.w().progress.set_text(Some(&format!(
                            "{} de {} — {}",
                            listing::human_size(p.done),
                            listing::human_size(p.total),
                            p.current
                        )));
                    }
                    Msg::Done(r) => {
                        me.s().task = None;
                        me.w().progress_bar.set_reveal_child(false);
                        me.reload();
                        match r {
                            Ok(msg) => {
                                log::info!("osmf: tarefa ok: {msg}");
                                me.w().status.set_text(&msg);
                            }
                            // Cancelamento pedido pelo usuário não é erro: só a barra de status.
                            Err(e) if e.ends_with("operação cancelada") => {
                                log::info!("osmf: tarefa falhou: {e}");
                                me.w().status.set_text(
                                    "Operação cancelada; nenhum arquivo parcial ficou no destino.",
                                );
                            }
                            Err(e) => {
                                log::warn!("osmf: tarefa falhou: {e}");
                                me.error(&e);
                            }
                        }
                        return glib::ControlFlow::Break;
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }

    fn error(&self, msg: &str) {
        log::warn!("osmf: erro: {msg}");
        self.w().status.set_text(msg);
        self.w().status.add_css_class("error");
        crate::dialogs::message(&self.w().window, "Não foi possível concluir", msg);
    }

    fn info(&self, msg: &str) {
        log::info!("osmf: aviso: {msg}");
        crate::dialogs::message(&self.w().window, "OSMarcio Files", msg);
    }
}

fn grid_factory() -> gtk4::SignalListItemFactory {
    let f = gtk4::SignalListItemFactory::new();
    f.connect_setup(|_, li| {
        let li = li.downcast_ref::<gtk4::ListItem>().unwrap();
        let b = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        b.set_margin_top(6);
        b.set_margin_bottom(6);
        let img = gtk4::Image::new();
        img.set_pixel_size(48);
        b.append(&img);
        b.append(
            &gtk4::Label::builder()
                .wrap(true)
                .wrap_mode(gtk4::pango::WrapMode::WordChar)
                .lines(2)
                .ellipsize(gtk4::pango::EllipsizeMode::Middle)
                .max_width_chars(14)
                .justify(gtk4::Justification::Center)
                .build(),
        );
        li.set_child(Some(&b));
    });
    f.connect_bind(|_, li| {
        let li = li.downcast_ref::<gtk4::ListItem>().unwrap();
        let (Some(obj), Some(b)) = (li.item(), li.child()) else {
            return;
        };
        let e = obj
            .downcast_ref::<glib::BoxedAnyObject>()
            .unwrap()
            .borrow::<Entry>()
            .clone();
        let img = b.first_child().and_downcast::<gtk4::Image>().unwrap();
        match &e.icon {
            Some(i) => img.set_from_gicon(i),
            None => img.set_icon_name(Some("text-x-generic")),
        }
        b.last_child()
            .and_downcast::<gtk4::Label>()
            .unwrap()
            .set_text(&e.name);
        b.set_tooltip_text(Some(&e.path.display().to_string()));
        b.set_widget_name(&format!("osmf-pos-{}", li.position()));
    });
    f
}

/// Favoritos do GTK: linhas "file:///caminho Nome opcional".
fn bookmarks() -> Vec<(PathBuf, String)> {
    let Ok(dir) = osmarcio_core::config::config_dir() else {
        return Vec::new();
    };
    let file = dir.parent().unwrap_or(&dir).join("gtk-3.0/bookmarks");
    std::fs::read_to_string(file)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (uri, name) = l.split_once(' ').unwrap_or((l, ""));
            let p = gio::File::for_uri(uri).path()?;
            let n = if name.is_empty() {
                p.file_name()?.to_string_lossy().into_owned()
            } else {
                name.to_string()
            };
            Some((p, n))
        })
        .collect()
}
