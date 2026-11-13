//! Política de atualização do OSMarcio Browser — lógica pura (sem rede, sem CEF).
//!
//! O runtime Chromium/CEF é parte do **pacote** `osmarcio-browser` (binário Rust
//! compilado contra uma versão exata do CEF). Por isso uma "atualização do
//! Chromium" é sempre uma nova versão do pacote OSMarcio Browser, homologada e
//! distribuída por um canal assinado (APT). Este módulo só **decide o estado**
//! a partir de dados já coletados; quem coleta (dpkg, apt-cache, curl) é o app.
//!
//! Fontes, em ordem:
//! 1. pacote instalado mais novo que o processo em execução → reinício necessário;
//! 2. candidato do APT (repositório assinado, verificado pelo próprio APT);
//! 3. manifesto de releases homologadas (`manifest_url` em
//!    `/etc/osmarcio/browser-updates.toml`) — informativo: nunca baixa nem executa nada.

use serde::Deserialize;
use std::cmp::Ordering;

/// Arquivo de configuração do canal de atualizações (conffile do pacote).
pub const CONFIG_PATH: &str = "/etc/osmarcio/browser-updates.toml";

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct UpdateConfig {
    /// URL do manifesto TOML de releases homologadas (`https://` ou `file://`).
    #[serde(default)]
    pub manifest_url: String,
}

impl UpdateConfig {
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| format!("configuração inválida: {e}"))
    }

    /// Só HTTPS (certificado verificado) ou arquivo local do administrador.
    pub fn validate_url(&self) -> Result<Option<&str>, String> {
        let u = self.manifest_url.trim();
        if u.is_empty() {
            Ok(None)
        } else if u.starts_with("https://") || u.starts_with("file:///") {
            Ok(Some(u))
        } else {
            Err(format!(
                "manifest_url recusada (use https:// ou file:///): {u}"
            ))
        }
    }
}

/// Uma release homologada (ou não) do pacote OSMarcio Browser.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Release {
    pub version: String,
    /// Arquitetura Debian: arm64, amd64.
    pub arch: String,
    #[serde(default)]
    pub chromium: String,
    #[serde(default)]
    pub cef: String,
    /// Só releases homologadas (testadas pelo projeto) são oferecidas.
    #[serde(default)]
    pub homologated: bool,
    /// SHA-256 do .deb publicado (obrigatório para ser considerada).
    #[serde(default)]
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(default)]
    release: Vec<Release>,
}

pub fn parse_manifest(text: &str) -> Result<Vec<Release>, String> {
    toml::from_str::<Manifest>(text)
        .map(|m| m.release)
        .map_err(|e| format!("manifesto inválido: {e}"))
}

/// Compara versões Debian simples ("0.2.10" > "0.2.9"; "0.2.1-1" > "0.2.1").
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let parts = |v: &str| -> Vec<u64> {
        v.split(|c: char| !c.is_ascii_digit())
            .filter(|p| !p.is_empty())
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    let (pa, pb) = (parts(a), parts(b));
    for i in 0..pa.len().max(pb.len()) {
        match pa.get(i).unwrap_or(&0).cmp(pb.get(i).unwrap_or(&0)) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    Ordering::Equal
}

/// Estados mostrados ao usuário (tela Sobre e `--check-updates`).
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateState {
    /// Nada mais novo em nenhuma fonte configurada.
    UpToDate { installed: String },
    /// Versão homologada disponível pelo APT (assinado): instalar com `apt`.
    AvailableApt { version: String },
    /// Versão homologada listada no manifesto, mas sem canal APT para instalar.
    AvailableManifest { version: String, chromium: String },
    /// Há versão mais nova, ainda não homologada pelo projeto.
    NotHomologated { version: String },
    /// Há versão mais nova, mas não para esta arquitetura.
    Incompatible { version: String, arch: String },
    /// O pacote instalado em disco é mais novo que o processo em execução.
    RestartRequired { installed: String, running: String },
    /// Nenhuma fonte de atualização configurada.
    SourceNotConfigured,
    /// Falha ao consultar (rede, TLS, manifesto inválido...).
    CheckFailed(String),
}

impl UpdateState {
    pub fn message(&self) -> String {
        match self {
            Self::UpToDate { installed } => {
                format!("O OSMarcio Browser está atualizado ({installed}).")
            }
            Self::AvailableApt { version } => format!(
                "Atualização disponível: {version} (repositório APT assinado). \
                 Instale com: sudo apt install --only-upgrade osmarcio-browser"
            ),
            Self::AvailableManifest { version, chromium } => format!(
                "Versão homologada {version} (Chromium {chromium}) publicada, mas não há \
                 repositório APT assinado configurado para instalá-la com segurança."
            ),
            Self::NotHomologated { version } => {
                format!(
                    "Existe a versão {version}, ainda não homologada para o OSMarcio Linux — não oferecida."
                )
            }
            Self::Incompatible { version, arch } => {
                format!("A versão {version} não tem pacote para esta arquitetura ({arch}).")
            }
            Self::RestartRequired { installed, running } => format!(
                "Reinício necessário: o pacote {installed} já está instalado; esta janela ainda roda {running}."
            ),
            Self::SourceNotConfigured => {
                "Fonte de atualização não configurada (nem repositório APT do \
                 OSMarcio Browser nem manifest_url em /etc/osmarcio/browser-updates.toml)."
                    .to_string()
            }
            Self::CheckFailed(why) => format!("Não foi possível verificar atualizações: {why}"),
        }
    }
}

/// Decide o estado a partir do manifesto (sem rede).
pub fn evaluate_manifest(installed: &str, arch: &str, releases: &[Release]) -> UpdateState {
    let newer = |r: &&Release| compare_versions(&r.version, installed) == Ordering::Greater;
    let newest = |v: Vec<&Release>| {
        v.into_iter()
            .max_by(|a, b| compare_versions(&a.version, &b.version))
            .cloned()
    };

    let ok: Vec<&Release> = releases
        .iter()
        .filter(newer)
        .filter(|r| r.arch == arch && r.homologated && is_sha256(&r.sha256))
        .collect();
    if let Some(r) = newest(ok) {
        return UpdateState::AvailableManifest {
            version: r.version,
            chromium: r.chromium,
        };
    }
    if let Some(r) = newest(
        releases
            .iter()
            .filter(newer)
            .filter(|r| r.arch == arch)
            .collect(),
    ) {
        return UpdateState::NotHomologated { version: r.version };
    }
    if let Some(r) = newest(releases.iter().filter(newer).collect()) {
        return UpdateState::Incompatible {
            version: r.version,
            arch: arch.to_string(),
        };
    }
    UpdateState::UpToDate {
        installed: installed.to_string(),
    }
}

/// Lê `apt-cache policy <pacote>` (LC_ALL=C ou qualquer idioma): versão
/// instalada, candidata e se o candidato vem de um **repositório** de verdade.
/// Um pacote instalado de um .deb local aparece só com a origem
/// `/var/lib/dpkg/status` — isso NÃO é uma fonte de atualização.
pub fn parse_apt_policy(text: &str) -> AptPolicy {
    let mut vals = text
        .lines()
        .skip(1)
        .take(2)
        .map(|l| l.split_once(':').map(|(_, v)| v.trim().to_string()))
        .map(|v| v.filter(|s| !s.is_empty() && !s.starts_with('(')));
    let installed = vals.next().flatten();
    let candidate = vals.next().flatten();
    let from_repository = text.lines().any(|l| {
        let mut w = l.split_whitespace();
        matches!((w.next(), w.next()), (Some(p), Some(src))
            if p.chars().all(|c| c.is_ascii_digit()) && src.contains("://"))
    });
    AptPolicy {
        installed,
        candidate,
        from_repository,
    }
}

#[derive(Debug, PartialEq)]
pub struct AptPolicy {
    pub installed: Option<String>,
    pub candidate: Option<String>,
    /// Há ao menos uma origem `http(s)://`/`file:/`… (repositório APT configurado).
    pub from_repository: bool,
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "8780d9b0a87dd50bbc3a39c2c017f1a17c11e393c70d77ddd954dce67faa3e62";

    fn rel(v: &str, arch: &str, homologated: bool) -> Release {
        Release {
            version: v.into(),
            arch: arch.into(),
            chromium: "157.0.1.2".into(),
            cef: "157.0.1".into(),
            homologated,
            sha256: SHA.into(),
        }
    }

    #[test]
    fn versions_compare_numerically() {
        assert_eq!(compare_versions("0.2.10", "0.2.9"), Ordering::Greater);
        assert_eq!(compare_versions("0.2.1", "0.2.1"), Ordering::Equal);
        assert_eq!(compare_versions("0.2.1-1", "0.2.1"), Ordering::Greater);
        assert_eq!(compare_versions("0.1.9", "0.2"), Ordering::Less);
    }

    #[test]
    fn up_to_date_when_nothing_newer() {
        let r = [rel("0.2.1", "arm64", true), rel("0.1.0", "arm64", true)];
        assert_eq!(
            evaluate_manifest("0.2.1", "arm64", &r),
            UpdateState::UpToDate {
                installed: "0.2.1".into()
            }
        );
    }

    #[test]
    fn homologated_newer_same_arch_is_available() {
        let r = [rel("0.3.0", "arm64", true), rel("0.2.2", "arm64", true)];
        assert!(
            matches!(evaluate_manifest("0.2.1", "arm64", &r), UpdateState::AvailableManifest { version, .. } if version == "0.3.0")
        );
    }

    #[test]
    fn not_homologated_is_never_offered() {
        let r = [rel("0.3.0", "arm64", false)];
        assert_eq!(
            evaluate_manifest("0.2.1", "arm64", &r),
            UpdateState::NotHomologated {
                version: "0.3.0".into()
            }
        );
    }

    #[test]
    fn missing_checksum_is_not_offered() {
        let mut r = rel("0.3.0", "arm64", true);
        r.sha256 = "abc".into();
        assert_eq!(
            evaluate_manifest("0.2.1", "arm64", &[r]),
            UpdateState::NotHomologated {
                version: "0.3.0".into()
            }
        );
    }

    #[test]
    fn other_arch_is_incompatible() {
        let r = [rel("0.3.0", "amd64", true)];
        assert_eq!(
            evaluate_manifest("0.2.1", "arm64", &r),
            UpdateState::Incompatible {
                version: "0.3.0".into(),
                arch: "arm64".into()
            }
        );
    }

    #[test]
    fn manifest_parses_and_rejects_garbage() {
        let m = format!(
            "[[release]]\nversion = \"0.3.0\"\narch = \"arm64\"\nchromium = \"157\"\nhomologated = true\nsha256 = \"{SHA}\"\n"
        );
        assert_eq!(parse_manifest(&m).unwrap().len(), 1);
        assert!(parse_manifest("<html>not toml").is_err());
    }

    #[test]
    fn config_only_accepts_https_or_local_file() {
        let c = |u: &str| UpdateConfig {
            manifest_url: u.into(),
        };
        assert_eq!(c("").validate_url(), Ok(None));
        assert!(
            c("https://x.invalid/m.toml")
                .validate_url()
                .unwrap()
                .is_some()
        );
        assert!(
            c("file:///etc/osmarcio/m.toml")
                .validate_url()
                .unwrap()
                .is_some()
        );
        assert!(c("http://x.invalid/m.toml").validate_url().is_err());
        assert_eq!(
            UpdateConfig::parse("manifest_url = \"\"\n").unwrap(),
            UpdateConfig::default()
        );
    }

    #[test]
    fn apt_policy_parsing() {
        let repo = "osmarcio-browser:\n  Instalado: 0.2.1\n  Candidato: 0.2.2\n  Tabela de versão:\n     0.2.2 500\n        500 https://apt.example.invalid trixie/main arm64 Packages\n *** 0.2.1 100\n        100 /var/lib/dpkg/status\n";
        assert_eq!(
            parse_apt_policy(repo),
            AptPolicy {
                installed: Some("0.2.1".into()),
                candidate: Some("0.2.2".into()),
                from_repository: true
            }
        );
        // Instalado de um .deb local: candidato existe, mas não há repositório.
        let local = "osmarcio-browser:\n  Installed: 0.2.1\n  Candidate: 0.2.1\n  Version table:\n *** 0.2.1 100\n        100 /var/lib/dpkg/status\n";
        assert!(!parse_apt_policy(local).from_repository);
        let none = "osmarcio-browser:\n  Installed: (none)\n  Candidate: (none)\n";
        assert_eq!(
            parse_apt_policy(none),
            AptPolicy {
                installed: None,
                candidate: None,
                from_repository: false
            }
        );
        assert!(!parse_apt_policy("").from_repository);
    }
}
