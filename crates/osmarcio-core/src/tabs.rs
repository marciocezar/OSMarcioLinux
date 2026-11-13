//! Regras de abas compartilhadas (terminal, editor, browser).
//!
//! Regra da última aba (requisito do OSMarcio Terminal):
//! - com **uma** aba, o botão X da aba fica oculto e o atalho de fechar aba
//!   não a fecha;
//! - com **duas ou mais**, os controles de fechamento reaparecem.

/// O que fazer quando o usuário pede para fechar uma aba pelo teclado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseShortcut {
    /// Fecha a aba atual.
    CloseTab,
    /// Não fecha nada; a tecla é repassada ao conteúdo (no terminal, o
    /// Ctrl+W volta a ser o "apagar palavra" do readline/Bash).
    PassThrough,
}

/// Política de abas: puramente função da quantidade de abas abertas.
#[derive(Debug, Clone, Copy, Default)]
pub struct TabPolicy;

impl TabPolicy {
    /// O botão X das abas deve ser exibido?
    pub fn show_close_buttons(self, tab_count: usize) -> bool {
        tab_count > 1
    }

    /// Uma aba pode ser fechada (por botão, menu ou atalho)?
    pub fn can_close(self, tab_count: usize) -> bool {
        tab_count > 1
    }

    /// Decisão para o atalho de fechar aba (Ctrl+W).
    pub fn on_close_shortcut(self, tab_count: usize) -> CloseShortcut {
        if self.can_close(tab_count) {
            CloseShortcut::CloseTab
        } else {
            CloseShortcut::PassThrough
        }
    }

    /// "Fechar outras abas": índices a fechar, em ordem decrescente para que
    /// a remoção não invalide os índices seguintes.
    pub fn others_to_close(self, tab_count: usize, keep: usize) -> Vec<usize> {
        (0..tab_count).rev().filter(|&i| i != keep).collect()
    }

    /// Índice da próxima aba (Ctrl+Tab), com volta ao início.
    pub fn next(self, tab_count: usize, current: usize) -> usize {
        if tab_count == 0 {
            0
        } else {
            (current + 1) % tab_count
        }
    }

    /// Índice da aba anterior (Ctrl+Shift+Tab), com volta ao fim.
    pub fn previous(self, tab_count: usize, current: usize) -> usize {
        if tab_count == 0 {
            0
        } else {
            (current + tab_count - 1) % tab_count
        }
    }
}

/// Normaliza o nome digitado ao renomear uma aba. `None` = restaurar o
/// título automático (vindo do shell).
pub fn normalize_tab_title(input: &str) -> Option<String> {
    let t: String = input
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(64)
        .collect();
    if t.is_empty() { None } else { Some(t) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: TabPolicy = TabPolicy;

    #[test]
    fn last_tab_is_protected() {
        assert!(!P.show_close_buttons(1));
        assert!(!P.can_close(1));
        assert_eq!(P.on_close_shortcut(1), CloseShortcut::PassThrough);
    }

    #[test]
    fn controls_reappear_with_second_tab() {
        assert!(P.show_close_buttons(2));
        assert!(P.can_close(2));
        assert_eq!(P.on_close_shortcut(2), CloseShortcut::CloseTab);
    }

    #[test]
    fn close_others_keeps_selected_and_is_descending() {
        assert_eq!(P.others_to_close(4, 1), vec![3, 2, 0]);
        assert_eq!(P.others_to_close(1, 0), Vec::<usize>::new());
    }

    #[test]
    fn tab_cycling_wraps() {
        assert_eq!(P.next(3, 2), 0);
        assert_eq!(P.previous(3, 0), 2);
        assert_eq!(P.next(0, 0), 0);
    }

    #[test]
    fn titles_are_trimmed_and_limited() {
        assert_eq!(normalize_tab_title("  build  "), Some("build".into()));
        assert_eq!(normalize_tab_title("   "), None);
        assert_eq!(normalize_tab_title("a\u{7}b"), Some("ab".into()));
        assert_eq!(normalize_tab_title(&"x".repeat(100)).unwrap().len(), 64);
    }
}
