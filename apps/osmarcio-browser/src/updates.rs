//! Verificação de atualizações do OSMarcio Browser (coleta de dados).
//! A decisão fica em `osmarcio_core::updates` (testada sem rede).
//!
//! Segurança: este processo NUNCA instala nada nem roda como root. Ele só lê:
//! - `dpkg-query` (versão do pacote instalado);
//! - `apt-cache policy` (candidato vindo de repositório APT, cuja assinatura
//!   o próprio APT verifica no `apt update`);
//! - o manifesto de releases homologadas via `curl` com `--proto =https,file`
//!   (certificado TLS sempre verificado; nada é baixado além do manifesto).
//!   A instalação, quando houver canal assinado, é feita pelo usuário com `apt`.

use osmarcio_core::updates::{self, UpdateConfig, UpdateState, compare_versions};
use std::cmp::Ordering;
use std::process::Command;

const PKG: &str = "osmarcio-browser";

fn run(cmd: &str, args: &[&str]) -> Option<(i32, String)> {
    let out = Command::new(cmd)
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .ok()?;
    Some((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    ))
}

fn installed_package() -> Option<String> {
    run("dpkg-query", &["-W", "-f=${Version}", PKG])
        .filter(|(c, v)| *c == 0 && !v.is_empty())
        .map(|(_, v)| v)
}

/// Mensagem legível para os códigos de saída do curl mais comuns.
fn curl_error(code: i32) -> String {
    match code {
        6 => "sem conexão (nome do servidor não resolvido)".into(),
        7 | 28 => "sem conexão com o servidor de atualizações".into(),
        35 | 51 | 53 | 54 | 58 | 59 | 60 | 77 | 83 | 90 | 91 => {
            "falha TLS/certificado — a fonte não foi aceita".into()
        }
        1 => "protocolo recusado (só https:// ou file:///)".into(),
        22 => "o servidor respondeu com erro HTTP".into(),
        37 => "manifesto local não encontrado".into(),
        c => format!("curl terminou com código {c}"),
    }
}

pub fn check(running: &str, deb_arch: &str) -> UpdateState {
    // 1. Pacote já atualizado em disco, processo antigo em execução.
    let installed = installed_package();
    if let Some(inst) = &installed
        && compare_versions(inst, running) == Ordering::Greater
    {
        return UpdateState::RestartRequired {
            installed: inst.clone(),
            running: running.into(),
        };
    }
    let base = installed.clone().unwrap_or_else(|| running.to_string());

    // 2. Repositório APT (assinado) que publique o pacote.
    let mut apt_configured = false;
    if let Some((0, policy)) = run("apt-cache", &["policy", PKG]) {
        let p = updates::parse_apt_policy(&policy);
        // Só conta como fonte se o candidato vier de um repositório APT
        // (não do próprio /var/lib/dpkg/status de um .deb instalado à mão).
        if p.from_repository {
            apt_configured = true;
            if let Some(c) = p.candidate
                && compare_versions(&c, &base) == Ordering::Greater
            {
                return UpdateState::AvailableApt { version: c };
            }
        }
    }

    // 3. Manifesto de releases homologadas (informativo).
    let cfg = match std::fs::read_to_string(updates::CONFIG_PATH) {
        Ok(t) => match UpdateConfig::parse(&t) {
            Ok(c) => c,
            Err(e) => return UpdateState::CheckFailed(e),
        },
        Err(_) => UpdateConfig::default(),
    };
    let url = match cfg.validate_url() {
        Ok(Some(u)) => u.to_string(),
        Ok(None) if apt_configured => return UpdateState::UpToDate { installed: base },
        Ok(None) => return UpdateState::SourceNotConfigured,
        Err(e) => return UpdateState::CheckFailed(e),
    };
    log::info!("osmb: verificando atualizações em {url}");
    let Some((code, body)) = run(
        "curl",
        &[
            "-fsS",
            "--proto",
            "=https,file",
            "--max-time",
            "20",
            "--max-filesize",
            "1048576",
            &url,
        ],
    ) else {
        return UpdateState::CheckFailed("curl não encontrado".into());
    };
    if code != 0 {
        return UpdateState::CheckFailed(curl_error(code));
    }
    match updates::parse_manifest(&body) {
        Ok(rel) => updates::evaluate_manifest(&base, deb_arch, &rel),
        Err(e) => UpdateState::CheckFailed(e),
    }
}
