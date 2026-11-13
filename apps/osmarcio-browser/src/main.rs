//! OSMarcio Browser — navegador em Rust com motor Chromium via CEF.
//!
//! Modelo de processos (padrão do Chromium/CEF): este mesmo executável é
//! iniciado várias vezes. A primeira instância é o *browser process* (a
//! interface em Rust); o CEF relança o executável com `--type=renderer`,
//! `--type=gpu-process`, `--type=utility`… para os subprocessos, que entram
//! por `execute_process` e terminam ali.

mod about;
mod app;
mod client;
mod ui;
mod updates;
mod version;

use cef::*;

fn main() -> std::process::ExitCode {
    // PRIMEIRA chamada ao CEF: registra a versão da API C com que fomos
    // compilados. Qualquer outra chamada antes disto (até criar a linha de
    // comando) aborta o processo com CHECK — erro real encontrado no teste.
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    let args = args::Args::new();
    let Some(cmd_line) = args.as_cmd_line() else {
        eprintln!("osmarcio-browser: falha ao ler a linha de comando");
        return std::process::ExitCode::FAILURE;
    };

    let is_browser_process = cmd_line.has_switch(Some(&CefString::from("type"))) != 1;
    let code = execute_process(Some(args.as_main_args()), None, std::ptr::null_mut());
    if !is_browser_process {
        // Subprocesso (renderer, GPU, utility, zygote): já executou e terminou.
        return std::process::ExitCode::from(code.clamp(0, 255) as u8);
    }

    // Opções de linha de comando que não abrem janela (não inicializam o CEF;
    // o libcef.so já está carregado, então as versões são as do runtime real).
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "uso: osmarcio-browser [URL] [--about] [--version] [--check-updates] [--help]\n\
                  \n  --about          abre também a janela Sobre o OSMarcio Browser\
                  \n  --version        versões reais: Browser, Chromium, CEF, cef-rs, arquitetura, sistema\
                  \n  --check-updates  consulta as fontes de atualização configuradas (não instala nada)"
        );
        return std::process::ExitCode::SUCCESS;
    }
    if argv.iter().any(|a| a == "--version") {
        print!("{}", version::VersionInfo::collect().as_text());
        return std::process::ExitCode::SUCCESS;
    }
    if argv.iter().any(|a| a == "--check-updates") {
        let v = version::VersionInfo::collect();
        let st = updates::check(&v.browser, v.deb_arch());
        println!("{}", st.message());
        let failed = matches!(st, osmarcio_core::updates::UpdateState::CheckFailed(_));
        return if failed {
            std::process::ExitCode::from(2)
        } else {
            std::process::ExitCode::SUCCESS
        };
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!(
        "osmb: OSMarcio Browser {} (CEF {})",
        osmarcio_core::VERSION,
        cef_version()
    );

    if is_root() {
        eprintln!(
            "osmarcio-browser: não execute o navegador como root (o sandbox do Chromium exige usuário comum)."
        );
        return std::process::ExitCode::from(1);
    }

    let mut app = app::OsmApp::new();
    let settings = app::settings();
    if initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(&mut app),
        std::ptr::null_mut(),
    ) != 1
    {
        eprintln!(
            "osmarcio-browser: falha ao inicializar o CEF (veja ~/.cache/osmarcio/browser/cef.log)"
        );
        return std::process::ExitCode::FAILURE;
    }
    run_message_loop();
    shutdown();
    log::info!("osmb: encerrado");
    std::process::ExitCode::SUCCESS
}

/// Versão do CEF/Chromium com que o binário foi compilado (constantes do cef-dll-sys).
pub fn cef_version() -> String {
    let v = String::from_utf8_lossy(sys::CEF_VERSION);
    v.trim_end_matches('\0').to_string()
}

/// UID efetivo via /proc (sem `unsafe`/libc).
fn is_root() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(2).map(|u| u == "0"))
        })
        .unwrap_or(false)
}
