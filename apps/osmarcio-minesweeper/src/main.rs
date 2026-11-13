//! OSMarcio Minesweeper (Campo Minado) — interface GTK4.
//!
//! A interface só traduz eventos (clique, tecla, temporizador) em chamadas a
//! `game::Game` e redesenha o tabuleiro a partir do estado do jogo. Nenhuma
//! regra mora nos callbacks.

use gtk4::prelude::*;
use gtk4::{gio, glib};
use osmarcio_minesweeper::game::{Config, Difficulty, Game, Outcome, Rng, State};
use osmarcio_minesweeper::prefs::{self, Prefs};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

const APP_ID: &str = "org.osmarcio.Minesweeper";
const CSS: &str = "
.ms-board { padding: 8px; }
.ms-cell { min-width: 26px; min-height: 26px; padding: 0; margin: 0; border-radius: 3px; font-weight: bold; }
.ms-cell { background: #7d93ab; color: #ffffff; border: 1px solid #5f7690; }
.ms-cell:hover { background: #8fa6bf; }
.ms-cell:focus-visible { outline: 2px solid #f59e0b; outline-offset: -2px; }
.ms-cell.open { background: #eef1f4; color: #1f2937; border: 1px solid #d5dbe2; box-shadow: none; }
.ms-cell.boom { background: #c0392b; color: white; }
.ms-cell.wrong { background: #7f8c8d; }
.ms-cell.open.n1 { color: #2563eb; } .ms-cell.open.n2 { color: #15803d; } .ms-cell.open.n3 { color: #dc2626; }
.ms-cell.open.n4 { color: #7c3aed; } .ms-cell.open.n5 { color: #b45309; } .ms-cell.open.n6 { color: #0891b2; }
.ms-cell.open.n7 { color: #db2777; } .ms-cell.open.n8 { color: #4b5563; }
.ms-status { font-size: 1.1em; font-weight: bold; padding: 0 12px; }
";

struct Ui {
    window: gtk4::ApplicationWindow,
    grid: gtk4::Grid,
    cells: Vec<gtk4::Button>,
    mines_label: gtk4::Label,
    time_label: gtk4::Label,
    face: gtk4::Button,
    status: gtk4::Label,
}

struct App {
    game: Game,
    prefs: Prefs,
    ui: Option<Ui>,
    started: Option<Instant>,
    elapsed: u64,
    timer: Option<glib::SourceId>,
    test_seed: Option<u64>,
}

type Shared = Rc<RefCell<App>>;

fn rng(test_seed: Option<u64>) -> Rng {
    test_seed.map(Rng::new).unwrap_or_else(Rng::from_entropy)
}

fn main() -> glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    // Semente fixa só para testes automatizados (tabuleiro reproduzível).
    let test_seed = std::env::var("OSMARCIO_MINESWEEPER_SEED")
        .ok()
        .and_then(|s| s.parse().ok());
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| {
        let prefs: Prefs = osmarcio_core::config::load_or_default(prefs::APP);
        let cfg = prefs.config();
        let state = Rc::new(RefCell::new(App {
            game: Game::new(cfg, rng(test_seed)),
            prefs,
            ui: None,
            started: None,
            elapsed: 0,
            timer: None,
            test_seed,
        }));
        build(app, &state);
    });
    app.run_with_args::<&str>(&[])
}

fn build(app: &gtk4::Application, st: &Shared) {
    let css = gtk4::CssProvider::new();
    css.load_from_string(CSS);
    if let Some(d) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &d,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Campo Minado")
        .icon_name("osmarcio-minesweeper")
        .resizable(false)
        .build();

    // Menu do jogo.
    let menu = gio::Menu::new();
    menu.append(Some("Novo jogo"), Some("win.new"));
    let levels = gio::Menu::new();
    levels.append(Some("Fácil (9×9, 10 minas)"), Some("win.level::easy"));
    levels.append(
        Some("Intermediário (16×16, 40 minas)"),
        Some("win.level::intermediate"),
    );
    levels.append(Some("Difícil (30×16, 99 minas)"), Some("win.level::expert"));
    levels.append(Some("Personalizado…"), Some("win.custom"));
    menu.append_section(Some("Dificuldade"), &levels);
    let more = gio::Menu::new();
    more.append(Some("Recordes"), Some("win.records"));
    more.append(Some("Sobre o Campo Minado"), Some("win.about"));
    menu.append_section(None, &more);
    let menu_btn = gtk4::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .tooltip_text("Menu")
        .build();

    let header = gtk4::HeaderBar::new();
    header.pack_end(&menu_btn);
    window.set_titlebar(Some(&header));

    // Barra superior: minas restantes · reiniciar · tempo.
    let mines_label = gtk4::Label::new(None);
    mines_label.add_css_class("ms-status");
    let time_label = gtk4::Label::new(None);
    time_label.add_css_class("ms-status");
    let face = gtk4::Button::with_label("Novo jogo");
    face.set_tooltip_text(Some("Novo jogo (F2)"));
    let top = gtk4::CenterBox::new();
    top.set_start_widget(Some(&mines_label));
    top.set_center_widget(Some(&face));
    top.set_end_widget(Some(&time_label));
    top.set_margin_top(6);

    let grid = gtk4::Grid::builder()
        .row_spacing(1)
        .column_spacing(1)
        .halign(gtk4::Align::Center)
        .build();
    grid.add_css_class("ms-board");
    let status = gtk4::Label::new(Some(
        "Clique esquerdo: revelar · direito (ou F): bandeira · clique num número: abrir vizinhas",
    ));
    status.set_margin_bottom(6);
    status.set_wrap(true);

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    vbox.append(&top);
    vbox.append(&grid);
    vbox.append(&status);
    window.set_child(Some(&vbox));

    st.borrow_mut().ui = Some(Ui {
        window: window.clone(),
        grid,
        cells: Vec::new(),
        mines_label,
        time_label,
        face: face.clone(),
        status,
    });

    // Ações.
    let add = |name: &str, f: Box<dyn Fn(&Shared)>| {
        let a = gio::SimpleAction::new(name, None);
        let s = st.clone();
        a.connect_activate(move |_, _| f(&s));
        window.add_action(&a);
    };
    add("new", Box::new(new_game));
    add("custom", Box::new(custom_dialog));
    add("records", Box::new(show_records));
    add("about", Box::new(about));
    let level = gio::SimpleAction::new("level", Some(glib::VariantTy::STRING));
    let s = st.clone();
    level.connect_activate(move |_, v| {
        let d = match v.and_then(|v| v.get::<String>()).as_deref() {
            Some("intermediate") => Difficulty::Intermediate,
            Some("expert") => Difficulty::Expert,
            _ => Difficulty::Easy,
        };
        if let Some(cfg) = Config::preset(d) {
            set_config(&s, cfg);
        }
    });
    window.add_action(&level);
    app.set_accels_for_action("win.new", &["F2", "<Control>n"]);
    app.set_accels_for_action("win.about", &["F1"]);

    let s = st.clone();
    face.connect_clicked(move |_| new_game(&s));

    // Teclado: setas movem o foco célula a célula (navegação explícita por
    // linha/coluna — a navegação direcional genérica do GTK depende da
    // geometria e falhava no tabuleiro); F ou M marca bandeira; Enter/Espaço
    // ativam o botão (revelar).
    let keys = gtk4::EventControllerKey::new();
    let s = st.clone();
    let w = window.clone();
    keys.connect_key_pressed(move |_, key, _, _| {
        use gtk4::gdk::Key;
        let Some((r, c)) = gtk4::prelude::GtkWindowExt::focus(&w).and_then(|f| cell_of(&f)) else {
            return glib::Propagation::Proceed;
        };
        let (rows, cols) = {
            let a = s.borrow();
            (a.game.rows(), a.game.cols())
        };
        let target = match key {
            Key::Up | Key::KP_Up => Some((r.saturating_sub(1), c)),
            Key::Down | Key::KP_Down => Some(((r + 1).min(rows - 1), c)),
            Key::Left | Key::KP_Left => Some((r, c.saturating_sub(1))),
            Key::Right | Key::KP_Right => Some((r, (c + 1).min(cols - 1))),
            Key::f | Key::F | Key::m | Key::M => {
                flag(&s, r, c);
                return glib::Propagation::Stop;
            }
            _ => None,
        };
        if let Some((nr, nc)) = target {
            let b = s
                .borrow()
                .ui
                .as_ref()
                .and_then(|u| u.cells.get(nr * cols + nc).cloned());
            if let Some(b) = b {
                b.grab_focus();
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
    window.add_controller(keys);

    rebuild_board(st);
    window.present();
    log::info!("osmm: janela criada");
}

/// Coordenadas de uma célula a partir do nome do botão ("r,c").
fn cell_of(w: &gtk4::Widget) -> Option<(usize, usize)> {
    let name = w.widget_name();
    let (r, c) = name.split_once(',')?;
    Some((r.parse().ok()?, c.parse().ok()?))
}

fn rebuild_board(st: &Shared) {
    let (rows, cols, grid) = {
        let a = st.borrow();
        let ui = a.ui.as_ref().unwrap();
        (a.game.rows(), a.game.cols(), ui.grid.clone())
    };
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
    let mut cells = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            let b = gtk4::Button::new();
            b.add_css_class("ms-cell");
            b.set_widget_name(&format!("{r},{c}"));
            let s = st.clone();
            b.connect_clicked(move |_| left_click(&s, r, c));
            let right = gtk4::GestureClick::builder().button(3).build();
            let s = st.clone();
            right.connect_pressed(move |_, _, _, _| flag(&s, r, c));
            b.add_controller(right);
            grid.attach(&b, c as i32, r as i32, 1, 1);
            cells.push(b);
        }
    }
    if let Some(ui) = st.borrow_mut().ui.as_mut() {
        ui.cells = cells;
    }
    refresh(st);
    if let Some(first) = st
        .borrow()
        .ui
        .as_ref()
        .and_then(|u| u.cells.first().cloned())
    {
        first.grab_focus();
    }
}

fn left_click(st: &Shared, r: usize, c: usize) {
    let before = st.borrow().game.state();
    let out = {
        let mut a = st.borrow_mut();
        let revealed = a.game.cell(r, c).is_some_and(|x| x.revealed);
        if revealed {
            a.game.chord(r, c)
        } else {
            a.game.reveal(r, c)
        }
    };
    log::info!("osmm: revelar {r},{c} -> {out:?}");
    if before == State::NotStarted && st.borrow().game.state() != State::NotStarted {
        start_timer(st);
        if st.borrow().test_seed.is_some() {
            log_mines(st);
        }
    }
    match out {
        Outcome::Won => finish(st, true),
        Outcome::Exploded => finish(st, false),
        _ => {}
    }
    refresh(st);
}

fn flag(st: &Shared, r: usize, c: usize) {
    let changed = st.borrow_mut().game.toggle_flag(r, c);
    if changed {
        let left = st.borrow().game.mines_left();
        log::info!("osmm: bandeira {r},{c} minas_restantes={left}");
        refresh(st);
    }
}

fn log_mines(st: &Shared) {
    let a = st.borrow();
    let g = &a.game;
    let mines: Vec<String> = (0..g.rows())
        .flat_map(|r| (0..g.cols()).map(move |c| (r, c)))
        .filter(|&(r, c)| g.cell(r, c).is_some_and(|x| x.mine))
        .map(|(r, c)| format!("{r},{c}"))
        .collect();
    log::info!("osmm: minas(teste)={}", mines.join(";"));
}

fn start_timer(st: &Shared) {
    let s = st.clone();
    let id = glib::timeout_add_seconds_local(1, move || {
        let mut a = s.borrow_mut();
        if let Some(t0) = a.started {
            a.elapsed = t0.elapsed().as_secs().min(999);
        }
        let txt = format!("Tempo: {}", a.elapsed);
        if let Some(ui) = a.ui.as_ref() {
            ui.time_label.set_text(&txt);
        }
        glib::ControlFlow::Continue
    });
    let mut a = st.borrow_mut();
    a.started = Some(Instant::now());
    a.timer = Some(id);
}

fn stop_timer(st: &Shared) {
    let mut a = st.borrow_mut();
    if let Some(id) = a.timer.take() {
        id.remove();
    }
    if let Some(t0) = a.started.take() {
        a.elapsed = t0.elapsed().as_secs().min(999);
    }
}

fn finish(st: &Shared, won: bool) {
    stop_timer(st);
    let mut a = st.borrow_mut();
    let secs = a.elapsed;
    let d = a.game.config().difficulty();
    let msg = if won {
        let rec = a.prefs.record(d, secs);
        if rec {
            save(&a.prefs);
        }
        format!(
            "Vitória em {secs} s!{}",
            if rec { " Novo recorde." } else { "" }
        )
    } else {
        "Você encontrou uma mina. Fim de jogo.".to_string()
    };
    log::info!("osmm: fim vitoria={won} tempo={secs}");
    if let Some(ui) = a.ui.as_ref() {
        ui.status.set_text(&msg);
        ui.face.set_label(if won {
            "Venceu! Novo jogo"
        } else {
            "Perdeu — Novo jogo"
        });
    }
}

fn refresh(st: &Shared) {
    let a = st.borrow();
    let Some(ui) = a.ui.as_ref() else { return };
    let g = &a.game;
    for (i, b) in ui.cells.iter().enumerate() {
        let (r, c) = (i / g.cols(), i % g.cols());
        let cell = g.cell(r, c).copied().unwrap_or_default();
        for cls in [
            "open", "boom", "wrong", "n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8",
        ] {
            b.remove_css_class(cls);
        }
        let label = if cell.revealed {
            b.add_css_class("open");
            if cell.mine {
                if g.exploded() == Some((r, c)) {
                    b.add_css_class("boom");
                }
                "●".to_string()
            } else if cell.adjacent > 0 {
                b.add_css_class(&format!("n{}", cell.adjacent));
                cell.adjacent.to_string()
            } else {
                String::new()
            }
        } else if cell.flagged {
            if g.state() == State::Lost && !cell.mine {
                b.add_css_class("wrong");
                "✗".to_string()
            } else {
                "⚑".to_string()
            }
        } else {
            String::new()
        };
        b.set_label(&label);
        b.set_tooltip_text(None);
    }
    ui.mines_label
        .set_text(&format!("Minas: {}", g.mines_left()));
    ui.time_label.set_text(&format!("Tempo: {}", a.elapsed));
    let cfg = g.config();
    ui.window.set_title(Some(&format!(
        "Campo Minado — {}×{}, {} minas",
        cfg.cols, cfg.rows, cfg.mines
    )));
}

fn new_game(st: &Shared) {
    stop_timer(st);
    {
        let mut a = st.borrow_mut();
        a.elapsed = 0;
        let seed = a.test_seed;
        let cfg = a.game.config();
        a.game = Game::new(cfg, rng(seed));
        if let Some(ui) = a.ui.as_ref() {
            ui.face.set_label("Novo jogo");
            ui.status.set_text("Clique esquerdo: revelar · direito (ou F): bandeira · clique num número: abrir vizinhas");
        }
    }
    log::info!("osmm: novo jogo {:?}", st.borrow().game.config());
    refresh(st);
}

fn set_config(st: &Shared, cfg: Config) {
    {
        let mut a = st.borrow_mut();
        a.prefs.set_config(cfg);
        save(&a.prefs);
        let seed = a.test_seed;
        a.game = Game::new(cfg, rng(seed));
    }
    stop_timer(st);
    st.borrow_mut().elapsed = 0;
    rebuild_board(st);
    new_game(st);
}

fn save(p: &Prefs) {
    match osmarcio_core::config::config_path(prefs::APP) {
        Ok(path) => {
            if let Err(e) = osmarcio_core::config::save_to(&path, p) {
                log::warn!("osmm: não foi possível salvar preferências: {e}");
            }
        }
        Err(e) => log::warn!("osmm: diretório de configuração indisponível: {e}"),
    }
}

fn parent(st: &Shared) -> gtk4::ApplicationWindow {
    st.borrow().ui.as_ref().unwrap().window.clone()
}

fn custom_dialog(st: &Shared) {
    let cur = st.borrow().game.config();
    let win = gtk4::Window::builder()
        .title("Partida personalizada")
        .modal(true)
        .transient_for(&parent(st))
        .resizable(false)
        .build();
    let spin = |min: f64, max: f64, v: usize| {
        let s = gtk4::SpinButton::with_range(min, max, 1.0);
        s.set_value(v as f64);
        s
    };
    let rows = spin(5.0, 24.0, cur.rows);
    let cols = spin(5.0, 40.0, cur.cols);
    let mines = spin(1.0, 951.0, cur.mines);
    let err = gtk4::Label::new(None);
    err.add_css_class("error");
    err.set_wrap(true);
    let g = gtk4::Grid::builder()
        .row_spacing(8)
        .column_spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    for (i, (txt, w)) in [
        ("Linhas (5–24)", &rows),
        ("Colunas (5–40)", &cols),
        ("Minas", &mines),
    ]
    .into_iter()
    .enumerate()
    {
        let l = gtk4::Label::new(Some(txt));
        l.set_xalign(0.0);
        g.attach(&l, 0, i as i32, 1, 1);
        g.attach(w, 1, i as i32, 1, 1);
    }
    g.attach(&err, 0, 3, 2, 1);
    let ok = gtk4::Button::with_label("Jogar");
    ok.add_css_class("suggested-action");
    let cancel = gtk4::Button::with_label("Cancelar");
    let bb = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    bb.set_halign(gtk4::Align::End);
    bb.append(&cancel);
    bb.append(&ok);
    g.attach(&bb, 0, 4, 2, 1);
    win.set_child(Some(&g));
    let w = win.clone();
    cancel.connect_clicked(move |_| w.close());
    let (w, s) = (win.clone(), st.clone());
    ok.connect_clicked(move |_| {
        match Config::custom(
            rows.value() as usize,
            cols.value() as usize,
            mines.value() as usize,
        ) {
            Ok(cfg) => {
                w.close();
                set_config(&s, cfg);
            }
            Err(e) => err.set_text(&format!("Configuração inválida: {e}.")),
        }
    });
    win.present();
}

fn show_records(st: &Shared) {
    let p = st.borrow().prefs.clone();
    let f = |v: Option<u64>| v.map(|s| format!("{s} s")).unwrap_or_else(|| "—".into());
    let txt = format!(
        "Fácil: {}\nIntermediário: {}\nDifícil: {}\n\n(partidas personalizadas não contam)",
        f(p.best_easy),
        f(p.best_intermediate),
        f(p.best_expert)
    );
    gtk4::AlertDialog::builder()
        .message("Melhores tempos")
        .detail(txt)
        .buttons(["Fechar"])
        .build()
        .show(Some(&parent(st)));
}

fn about(st: &Shared) {
    let d = gtk4::AboutDialog::builder()
        .transient_for(&parent(st))
        .modal(true)
        .program_name("Campo Minado")
        .version(env!("CARGO_PKG_VERSION"))
        .comments("OSMarcio Minesweeper — jogo educacional em Rust + GTK4 do OSMarcio Linux.\nRegras separadas da interface e cobertas por testes unitários.")
        .logo_icon_name("osmarcio-minesweeper")
        .license_type(gtk4::License::Gpl30)
        .authors(["Projeto OSMarcio Linux"])
        .build();
    d.present();
}
