//! Lógica do OSMarcio Browser que não depende do CEF: interpretação da barra
//! de endereços, indicador de segurança, páginas internas (início e erro),
//! histórico e favoritos em arquivo. Testável em qualquer host.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Endereço interno da página inicial.
pub const HOME_URL: &str = "osmarcio://inicio";
/// Busca usada quando o texto digitado não é um endereço.
pub const SEARCH_URL: &str = "https://duckduckgo.com/?q=";

/// O que fazer com o texto digitado na barra de endereços.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Url(String),
    Search(String),
    Home,
    /// Recusado por segurança (ex.: `javascript:` digitado na barra).
    Rejected(String),
}

impl Target {
    /// URL final a carregar (a página inicial é gerada pelo app).
    pub fn to_url(&self) -> Option<String> {
        match self {
            Target::Url(u) => Some(u.clone()),
            Target::Search(q) => Some(format!("{SEARCH_URL}{}", url_encode(q))),
            Target::Home | Target::Rejected(_) => None,
        }
    }
}

pub fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn looks_like_host(s: &str) -> bool {
    let host = s.split(['/', '?', '#']).next().unwrap_or("");
    let host_no_port = host.rsplit_once(':').map_or(host, |(h, p)| {
        if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() {
            h
        } else {
            host
        }
    });
    if host_no_port.eq_ignore_ascii_case("localhost") {
        return true;
    }
    !host_no_port.is_empty()
        && host_no_port.contains('.')
        && !host_no_port.starts_with('.')
        && !host_no_port.ends_with('.')
        && host_no_port
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// Interpreta o texto da barra de endereços.
pub fn parse_input(input: &str) -> Target {
    let t = input.trim();
    if t.is_empty() {
        return Target::Rejected("endereço vazio".into());
    }
    let lower = t.to_ascii_lowercase();
    if lower == HOME_URL || lower == "about:home" || lower == "osmarcio:inicio" {
        return Target::Home;
    }
    if lower.starts_with("https://") || lower.starts_with("http://") || lower == "about:blank" {
        return Target::Url(t.to_string());
    }
    if let Some((scheme, _)) = lower.split_once(':')
        && !scheme.is_empty()
        && scheme.chars().all(|c| c.is_ascii_alphabetic())
        && !looks_like_host(t)
    {
        return Target::Rejected(format!(
            "esquema \"{scheme}:\" não é aceito na barra de endereços"
        ));
    }
    if !t.contains(char::is_whitespace) && looks_like_host(t) {
        return Target::Url(format!("https://{t}"));
    }
    Target::Search(t.to_string())
}

/// Indicador exibido ao lado da barra de endereços.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Security {
    Secure,
    Insecure,
    Internal,
}

impl Security {
    pub fn of(url: &str) -> Self {
        let l = url.to_ascii_lowercase();
        if l.starts_with("https://") {
            Security::Secure
        } else if l.starts_with("data:") || l.starts_with("about:") || l.is_empty() {
            Security::Internal
        } else {
            Security::Insecure
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Security::Secure => "🔒 HTTPS",
            Security::Insecure => "⚠ HTTP",
            Security::Internal => "OSMarcio",
        }
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

const STYLE: &str = "body{font-family:'DejaVu Sans',sans-serif;background:#f4f6fb;color:#1e2430;margin:0}\
main{max-width:760px;margin:48px auto;padding:0 24px}h1{color:#2b5fd9;margin-bottom:4px}\
.sub{color:#5c6680;margin-top:0}ul{padding-left:18px}li{margin:6px 0}a{color:#2b5fd9}\
.card{background:#fff;border-radius:10px;padding:16px 20px;margin:18px 0;box-shadow:0 1px 3px #0002}\
code{background:#e8ecf6;padding:2px 6px;border-radius:4px}";

/// Página inicial gerada localmente (favoritos + histórico recente).
pub fn home_page(version: &str, bookmarks: &[Entry], history: &[Entry]) -> String {
    let list = |items: &[Entry], empty: &str| -> String {
        if items.is_empty() {
            return format!("<p class=sub>{empty}</p>");
        }
        let li: String = items
            .iter()
            .map(|e| {
                let title = if e.title.is_empty() { &e.url } else { &e.title };
                format!(
                    "<li><a href=\"{}\">{}</a></li>",
                    html_escape(&e.url),
                    html_escape(title)
                )
            })
            .collect();
        format!("<ul>{li}</ul>")
    };
    format!(
        "<!doctype html><html lang=pt-BR><head><meta charset=utf-8><title>Início — OSMarcio Browser</title>\
<style>{STYLE}</style></head><body><main><h1>OSMarcio Browser</h1>\
<p class=sub>versão {v} · motor Chromium via CEF · interface em Rust</p>\
<div class=card><b>Favoritos</b> <span class=sub>(Ctrl+D adiciona a página atual)</span>{b}</div>\
<div class=card><b>Visitados recentemente</b>{h}</div>\
<div class=card><b>Atalhos</b><p>Ctrl+T nova aba · Ctrl+W fecha aba · Ctrl+Tab próxima aba · Ctrl+L barra de endereços · \
F5 recarregar · Alt+←/→ voltar/avançar · Ctrl+=/−/0 zoom · F11 tela cheia · Ctrl+Q sair</p></div>\
</main></body></html>",
        v = html_escape(version),
        b = list(bookmarks, "Nenhum favorito ainda."),
        h = list(history, "Nenhuma página visitada ainda."),
    )
}

/// Página de erro de navegação (DNS, certificado, conexão…).
pub fn error_page(url: &str, code: i32, text: &str) -> String {
    let hint = match text {
        t if t.contains("NAME_NOT_RESOLVED") => {
            "O nome do site não foi encontrado (DNS). Verifique o endereço e a conexão: <code>resolvectl query &lt;site&gt;</code>."
        }
        t if t.contains("CERT") => {
            "O certificado do site é inválido ou não confiável. Por segurança, a página <b>não</b> foi carregada e não há opção de ignorar o erro."
        }
        t if t.contains("INTERNET_DISCONNECTED") || t.contains("ADDRESS_UNREACHABLE") => {
            "Sem conexão de rede. Veja <code>ip route</code> e <code>nmcli device status</code>."
        }
        t if t.contains("TIMED_OUT") => "O servidor demorou demais para responder.",
        _ => "A página não pôde ser carregada.",
    };
    format!(
        "<!doctype html><html lang=pt-BR><head><meta charset=utf-8><title>Erro — {t}</title>\
<style>{STYLE}</style></head><body><main><h1>Não foi possível abrir a página</h1>\
<div class=card><p><b>Endereço:</b> {u}</p><p><b>Erro:</b> <code>{t}</code> ({code})</p><p>{hint}</p></div>\
</main></body></html>",
        u = html_escape(url),
        t = html_escape(text),
    )
}

/// Página interna como `data:` URL (base64), sem acesso a arquivos.
pub fn data_url(html: &str) -> String {
    format!(
        "data:text/html;charset=utf-8;base64,{}",
        base64(html.as_bytes())
    )
}

pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if c.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

// ------------------------------------------------------- histórico/favoritos

/// Linha de histórico ou favorito.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub url: String,
    pub title: String,
}

/// Arquivo TSV simples: `url<TAB>título` por linha, mais recente no fim.
pub struct EntryFile {
    path: PathBuf,
}

impl EntryFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// `$XDG_DATA_HOME/osmarcio/browser/<nome>.tsv`.
    pub fn in_data_dir(name: &str) -> Option<Self> {
        let d = directories::BaseDirs::new()?
            .data_dir()
            .join("osmarcio/browser");
        Some(Self::new(d.join(format!("{name}.tsv"))))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, e: &Entry) -> std::io::Result<()> {
        if let Some(d) = self.path.parent() {
            std::fs::create_dir_all(d)?;
        }
        let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(f, "{}\t{}", clean(&e.url), clean(&e.title))
    }

    /// Últimas `n` entradas, mais recentes primeiro, sem URLs repetidas.
    pub fn recent(&self, n: usize) -> Vec<Entry> {
        let text = std::fs::read_to_string(&self.path).unwrap_or_default();
        let mut seen = std::collections::HashSet::new();
        text.lines()
            .rev()
            .filter_map(|l| {
                let (u, t) = l.split_once('\t').unwrap_or((l, ""));
                (!u.is_empty() && seen.insert(u.to_string())).then(|| Entry {
                    url: u.into(),
                    title: t.into(),
                })
            })
            .take(n)
            .collect()
    }

    pub fn contains(&self, url: &str) -> bool {
        std::fs::read_to_string(&self.path)
            .unwrap_or_default()
            .lines()
            .any(|l| l.split('\t').next() == Some(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_bar_rules() {
        assert_eq!(
            parse_input("https://example.com"),
            Target::Url("https://example.com".into())
        );
        assert_eq!(
            parse_input("  example.com/x?y=1 "),
            Target::Url("https://example.com/x?y=1".into())
        );
        assert_eq!(
            parse_input("localhost:8080"),
            Target::Url("https://localhost:8080".into())
        );
        assert_eq!(
            parse_input("debian.org"),
            Target::Url("https://debian.org".into())
        );
        assert_eq!(
            parse_input("como instalar rust"),
            Target::Search("como instalar rust".into())
        );
        assert_eq!(parse_input("rust"), Target::Search("rust".into()));
        assert_eq!(parse_input("osmarcio://inicio"), Target::Home);
        assert!(matches!(
            parse_input("javascript:alert(1)"),
            Target::Rejected(_)
        ));
        assert!(matches!(
            parse_input("file:///etc/shadow"),
            Target::Rejected(_)
        ));
        assert!(matches!(
            parse_input("chrome://settings"),
            Target::Rejected(_)
        ));
        assert!(matches!(parse_input(""), Target::Rejected(_)));
        assert_eq!(
            parse_input("http://neverssl.com"),
            Target::Url("http://neverssl.com".into())
        );
    }

    #[test]
    fn search_is_encoded() {
        assert_eq!(
            Target::Search("a&b c/ç".into()).to_url().unwrap(),
            "https://duckduckgo.com/?q=a%26b+c%2F%C3%A7"
        );
    }

    #[test]
    fn security_indicator() {
        assert_eq!(Security::of("https://x.org"), Security::Secure);
        assert_eq!(Security::of("http://x.org"), Security::Insecure);
        assert_eq!(Security::of("data:text/html,x"), Security::Internal);
    }

    #[test]
    fn pages_escape_untrusted_text() {
        let p = error_page("https://x/<script>", -105, "ERR_NAME_NOT_RESOLVED");
        assert!(p.contains("&lt;script&gt;") && !p.contains("<script>"));
        assert!(p.contains("DNS"));
        let h = home_page(
            "0.2.0",
            &[Entry {
                url: "https://a\"b".into(),
                title: "<b>".into(),
            }],
            &[],
        );
        assert!(h.contains("&quot;") && h.contains("&lt;b&gt;"));
    }

    #[test]
    fn base64_matches_reference() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64("ção".as_bytes()), "w6fDo28=");
        assert!(data_url("<p>").starts_with("data:text/html;charset=utf-8;base64,"));
    }

    #[test]
    fn entry_file_recent_dedup() {
        let d = tempfile::tempdir().unwrap();
        let f = EntryFile::new(d.path().join("x/h.tsv"));
        for (u, t) in [
            ("https://a", "A"),
            ("https://b", "B\tx"),
            ("https://a", "A2"),
        ] {
            f.append(&Entry {
                url: u.into(),
                title: t.into(),
            })
            .unwrap();
        }
        let r = f.recent(10);
        assert_eq!(r.len(), 2);
        assert_eq!(
            r[0],
            Entry {
                url: "https://a".into(),
                title: "A2".into()
            }
        );
        assert_eq!(r[1].title, "B x");
        assert!(f.contains("https://b") && !f.contains("https://c"));
    }
}
