//! `osmarcio-info` — mostra informações reais do sistema.
//! Toda a coleta está em `osmarcio_core::sysinfo` (compartilhada com o
//! "Sobre o OSMarcio Linux").

use osmarcio_core::sysinfo::{SystemInfo, format_duration, format_kib};
use std::process::ExitCode;

const HELP: &str = "\
osmarcio-info — informações do sistema OSMarcio Linux

Uso: osmarcio-info [OPÇÃO]

  (sem opção)   resumo
  --version     versão da distribuição e do build
  --kernel      kernel Linux
  --hardware    CPU, memória, arquitetura
  --network     interfaces, gateway, DNS
  --desktop     sessão gráfica
  --all         todas as seções
  --json        todas as informações em JSON (para scripts)
  --help        esta ajuda

Fontes: /etc/os-release, /etc/debian_version, /usr/lib/osmarcio/build-info,
/proc/{cpuinfo,meminfo,uptime,sys/kernel}, `ip -j`, resolv.conf, variáveis XDG_*.
Valores indisponíveis aparecem como \"—\"; nada é inventado.";

fn v(o: &Option<String>) -> &str {
    o.as_deref().unwrap_or("—")
}

fn row(k: &str, val: impl AsRef<str>) {
    println!("{:<15}{}", format!("{k}:"), val.as_ref());
}

fn section(title: &str) {
    println!("\n[{title}]");
}

fn mem(i: &SystemInfo) -> String {
    match (i.mem_total_kib, i.mem_available_kib) {
        (Some(t), Some(a)) => format!("{} total, {} disponível", format_kib(t), format_kib(a)),
        (Some(t), None) => format_kib(t),
        _ => "—".into(),
    }
}

fn cpu(i: &SystemInfo) -> String {
    match (&i.cpu_model, i.cpu_count) {
        (Some(m), Some(n)) => format!("{m} ({n} CPU{})", if n > 1 { "s" } else { "" }),
        _ => "—".into(),
    }
}

fn display(i: &SystemInfo) -> String {
    match i.session_type.as_deref() {
        Some("x11") => "X11".into(),
        Some("wayland") => "Wayland".into(),
        Some("tty") => "console (tty)".into(),
        Some(o) => o.into(),
        None => "— (fora de uma sessão de login)".into(),
    }
}

fn build(i: &SystemInfo) -> String {
    let mut s = v(&i.build_id).to_string();
    if let Some(p) = i.build_info.get("PROFILE") {
        s.push_str(&format!(" · perfil {p}"));
    }
    if let Some(c) = i.build_info.get("COMMIT") {
        s.push_str(&format!(" · commit {c}"));
    }
    s
}

fn summary(i: &SystemInfo) {
    println!("OSMarcio Linux — System Information\n");
    row("OS Name", v(&i.os_name));
    row("OS Version", v(&i.os_version));
    row("Base", v(&i.base));
    row("Kernel", v(&i.kernel_release));
    row("Architecture", &i.arch);
    row("Hostname", v(&i.hostname));
    row("Desktop", v(&i.desktop));
    row("Display", display(i));
    row("CPU", cpu(i));
    row("Memory", mem(i));
    row(
        "Uptime",
        i.uptime.map(format_duration).unwrap_or_else(|| "—".into()),
    );
    row("Network", i.network_summary());
    row("Build", build(i));
}

fn version(i: &SystemInfo) {
    row("Distribuição", v(&i.os_pretty));
    row("ID", v(&i.os_id));
    row("Versão", v(&i.os_version));
    row("Variante", v(&i.os_variant));
    row("Build", build(i));
    if let Some(d) = i.build_info.get("BUILD_DATE") {
        row("Data do build", d);
    }
    row("Base", v(&i.base));
    row("osmarcio-info", osmarcio_core::VERSION);
}

fn kernel(i: &SystemInfo) {
    row("Release", v(&i.kernel_release));
    row("Build", v(&i.kernel_version));
    row("Arquitetura", &i.arch);
}

fn hardware(i: &SystemInfo) {
    row("Arquitetura", &i.arch);
    row("CPU", cpu(i));
    row("Memória", mem(i));
    row(
        "Uptime",
        i.uptime.map(format_duration).unwrap_or_else(|| "—".into()),
    );
}

fn network(i: &SystemInfo) {
    row("Estado", i.network_summary());
    for n in &i.interfaces {
        let a = if n.ipv4.is_empty() {
            "sem IPv4".to_string()
        } else {
            n.ipv4.join(", ")
        };
        row(&format!("  {}", n.name), format!("{} — {a}", n.state));
    }
    row(
        "Gateway",
        i.gateway
            .as_ref()
            .map(|(g, d)| format!("{g} ({d})"))
            .unwrap_or_else(|| "—".into()),
    );
    row(
        "DNS",
        if i.dns.is_empty() {
            "—".into()
        } else {
            i.dns.join(", ")
        },
    );
}

fn desktop(i: &SystemInfo) {
    row("Desktop", v(&i.desktop));
    row("Sessão", display(i));
}

fn json(i: &SystemInfo) -> String {
    let ifaces: Vec<_> = i
        .interfaces
        .iter()
        .map(|n| serde_json::json!({"name": n.name, "state": n.state, "ipv4": n.ipv4}))
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "os": {"name": i.os_name, "pretty_name": i.os_pretty, "id": i.os_id,
               "version": i.os_version, "variant": i.os_variant, "build_id": i.build_id,
               "base": i.base, "build_info": i.build_info},
        "kernel": {"release": i.kernel_release, "version": i.kernel_version},
        "arch": i.arch,
        "hostname": i.hostname,
        "desktop": {"name": i.desktop, "session_type": i.session_type},
        "cpu": {"model": i.cpu_model, "count": i.cpu_count},
        "memory_kib": {"total": i.mem_total_kib, "available": i.mem_available_kib},
        "uptime_seconds": i.uptime.map(|d| d.as_secs()),
        "network": {"interfaces": ifaces, "gateway": i.gateway.as_ref().map(|g| &g.0), "dns": i.dns},
    }))
    .unwrap_or_default()
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1).unwrap_or_default();
    if arg == "--help" || arg == "-h" {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let i = SystemInfo::collect();
    match arg.as_str() {
        "" => summary(&i),
        "--version" | "-V" => version(&i),
        "--kernel" => kernel(&i),
        "--hardware" => hardware(&i),
        "--network" => network(&i),
        "--desktop" => desktop(&i),
        "--json" => println!("{}", json(&i)),
        "--all" => {
            summary(&i);
            section("Versão");
            version(&i);
            section("Kernel");
            kernel(&i);
            section("Hardware");
            hardware(&i);
            section("Rede");
            network(&i);
            section("Desktop");
            desktop(&i);
        }
        other => {
            eprintln!("opção desconhecida: {other}\n\n{HELP}");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
