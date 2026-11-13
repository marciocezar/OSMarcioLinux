//! Diálogos do OSMarcio Files.

use crate::listing::{self, Entry};
use gtk4::prelude::*;

/// Pede um nome; `apply` devolve Err(mensagem) para manter o diálogo aberto.
pub fn ask_name(
    parent: &gtk4::ApplicationWindow,
    title: &str,
    initial: &str,
    apply: impl Fn(String) -> Result<(), String> + 'static,
) {
    let win = gtk4::Window::builder()
        .title(title)
        .modal(true)
        .transient_for(parent)
        .default_width(380)
        .resizable(false)
        .build();
    let entry = gtk4::Entry::builder()
        .text(initial)
        .activates_default(true)
        .build();
    let err = gtk4::Label::builder().xalign(0.0).wrap(true).build();
    err.add_css_class("error");
    let ok = gtk4::Button::with_label("OK");
    ok.add_css_class("suggested-action");
    let cancel = gtk4::Button::with_label("Cancelar");
    let bb = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    bb.set_halign(gtk4::Align::End);
    bb.append(&cancel);
    bb.append(&ok);
    let v = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    v.set_margin_top(14);
    v.set_margin_bottom(14);
    v.set_margin_start(14);
    v.set_margin_end(14);
    v.append(&entry);
    v.append(&err);
    v.append(&bb);
    win.set_child(Some(&v));
    win.set_default_widget(Some(&ok));
    let w = win.clone();
    cancel.connect_clicked(move |_| w.close());
    let (w, e) = (win.clone(), entry.clone());
    ok.connect_clicked(move |_| match apply(e.text().to_string()) {
        Ok(()) => w.close(),
        Err(msg) => err.set_text(&msg),
    });
    win.present();
    entry.grab_focus();
    // Seleciona só o nome, sem a extensão ("relatório|.txt"), DEPOIS do foco:
    // ao receber o foco o GtkEntry seleciona tudo, o que apagava a extensão.
    let stem = initial
        .rfind('.')
        .filter(|&i| i > 0)
        .map(|i| initial[..i].chars().count())
        .unwrap_or(initial.chars().count());
    // Síncrono, logo após o foco (um idle podia re-selecionar texto já digitado).
    entry.select_region(0, stem as i32);
}

pub fn confirm(
    parent: &gtk4::ApplicationWindow,
    title: &str,
    detail: &str,
    action: &str,
    yes: impl Fn() + 'static,
) {
    let d = gtk4::AlertDialog::builder()
        .message(title)
        .detail(detail)
        .buttons(["Cancelar", action])
        .cancel_button(0)
        .default_button(0)
        .modal(true)
        .build();
    d.choose(Some(parent), None::<&gtk4::gio::Cancellable>, move |r| {
        if r == Ok(1) {
            yes();
        }
    });
}

pub fn message(parent: &gtk4::ApplicationWindow, title: &str, detail: &str) {
    gtk4::AlertDialog::builder()
        .message(title)
        .detail(detail)
        .buttons(["Fechar"])
        .modal(true)
        .build()
        .show(Some(parent));
}

pub fn properties(parent: &gtk4::ApplicationWindow, e: &Entry) {
    let win = gtk4::Window::builder()
        .title(format!("Propriedades — {}", e.name))
        .transient_for(parent)
        .default_width(460)
        .build();
    let g = gtk4::Grid::builder()
        .row_spacing(6)
        .column_spacing(14)
        .margin_top(14)
        .margin_bottom(14)
        .margin_start(14)
        .margin_end(14)
        .build();
    let size = if e.is_dir {
        let n = std::fs::read_dir(&e.path).map(|r| r.count()).unwrap_or(0);
        let total = crate::fsops::total_size(
            std::slice::from_ref(&e.path),
            &gtk4::gio::Cancellable::new(),
        );
        format!("{n} itens · {} no total", listing::human_size(total))
    } else {
        format!("{} ({} bytes)", listing::human_size(e.size), e.size)
    };
    let mut rows = vec![
        ("Nome", e.name.clone()),
        ("Caminho", e.path.display().to_string()),
        ("Tipo", format!("{} ({})", e.description, e.content_type)),
        ("Tamanho", size),
        ("Modificado", listing::date(e.modified)),
        ("Permissões", listing::perms(e.mode)),
        (
            "Proprietário",
            if e.owner.is_empty() {
                "—".into()
            } else {
                e.owner.clone()
            },
        ),
    ];
    if let Some(t) = &e.link_target {
        rows.push(("Link simbólico para", t.clone()));
    }
    if let Some(t) = &e.trash {
        rows.push(("Local original", t.original.display().to_string()));
        rows.push(("Excluído em", t.deleted.clone()));
    }
    for (i, (k, v)) in rows.into_iter().enumerate() {
        let kl = gtk4::Label::builder().label(k).xalign(1.0).build();
        kl.add_css_class("dim-label");
        let vl = gtk4::Label::builder()
            .label(v)
            .xalign(0.0)
            .selectable(true)
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::WordChar)
            .build();
        g.attach(&kl, 0, i as i32, 1, 1);
        g.attach(&vl, 1, i as i32, 1, 1);
    }
    let close = gtk4::Button::with_label("Fechar");
    close.set_halign(gtk4::Align::End);
    let w = win.clone();
    close.connect_clicked(move |_| w.close());
    g.attach(&close, 1, 99, 1, 1);
    win.set_child(Some(&g));
    win.present();
    log::info!("osmf: propriedades {}", e.path.display());
}

pub fn about(parent: &gtk4::ApplicationWindow) {
    gtk4::AboutDialog::builder()
        .transient_for(parent)
        .modal(true)
        .program_name("OSMarcio Files")
        .version(env!("CARGO_PKG_VERSION"))
        .comments("Gerenciador de arquivos do OSMarcio Linux, em Rust + GTK4 + GIO.\nRoda como usuário comum e respeita as permissões do Linux.")
        .logo_icon_name("osmarcio-files")
        .license_type(gtk4::License::Gpl30)
        .authors(["Projeto OSMarcio Linux"])
        .build()
        .present();
}
