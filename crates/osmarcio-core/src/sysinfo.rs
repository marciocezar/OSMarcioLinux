//! Coleta de informações do sistema a partir de interfaces Linux reais
//! (`/etc/os-release`, `/proc`, `/sys`, `ip -j`). Usado por `osmarcio-info`
//! e pela janela "Sobre" — uma única implementação.
//!
//! Cada fonte tem um *parser puro* (testado com textos de exemplo) e uma
//! função de leitura que só abre o arquivo e chama o parser.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Campos de `/etc/os-release` (formato chave=valor com aspas opcionais).
pub fn parse_os_release(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return None;
            }
            let (k, v) = l.split_once('=')?;
            let v = v.trim();
            let v = v
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| v.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                .unwrap_or(v);
            Some((k.trim().to_string(), v.replace("\\\"", "\"")))
        })
        .collect()
}

/// Modelo de CPU e número de CPUs lógicas a partir de `/proc/cpuinfo`.
/// No ARM64 não há "model name": usa-se o *implementer/part* conhecido ou,
/// em VMs, o campo "CPU part".
pub fn parse_cpuinfo(text: &str) -> (String, usize) {
    let count = text.lines().filter(|l| l.starts_with("processor")).count();
    let field = |name: &str| {
        text.lines()
            .find(|l| l.split(':').next().map(str::trim) == Some(name))
            .and_then(|l| l.split_once(':'))
            .map(|(_, v)| v.trim().to_string())
    };
    let model = field("model name")
        .or_else(|| field("Model"))
        .or_else(|| {
            let imp = field("CPU implementer")?;
            let part = field("CPU part").unwrap_or_default();
            Some(arm_cpu_name(&imp, &part))
        })
        .unwrap_or_else(|| "desconhecida".into());
    (model, count.max(1))
}

fn arm_cpu_name(implementer: &str, part: &str) -> String {
    let vendor = match implementer {
        "0x41" => "ARM",
        "0x61" => "Apple",
        "0x51" => "Qualcomm",
        _ => "ARM64",
    };
    let core = match (implementer, part) {
        ("0x41", "0xd03") => "Cortex-A53",
        ("0x41", "0xd08") => "Cortex-A72",
        ("0x41", "0xd0b") => "Cortex-A76",
        ("0x41", "0xd0c") => "Neoverse-N1",
        _ => "",
    };
    if core.is_empty() {
        format!("{vendor} (implementer {implementer}, part {part})")
    } else {
        format!("{vendor} {core}")
    }
}

/// (total, disponível) em KiB a partir de `/proc/meminfo`.
pub fn parse_meminfo(text: &str) -> (u64, u64) {
    let get = |k: &str| {
        text.lines()
            .find(|l| l.starts_with(k))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    (get("MemTotal:"), get("MemAvailable:"))
}

/// Segundos de `/proc/uptime`.
pub fn parse_uptime(text: &str) -> Option<Duration> {
    let secs: f64 = text.split_whitespace().next()?.parse().ok()?;
    Some(Duration::from_secs_f64(secs))
}

/// "2 d 3 h 4 min" — formato legível.
pub fn format_duration(d: Duration) -> String {
    let s = d.as_secs();
    let (days, h, m) = (s / 86_400, (s % 86_400) / 3600, (s % 3600) / 60);
    match (days, h) {
        (0, 0) => format!("{m} min"),
        (0, _) => format!("{h} h {m} min"),
        _ => format!("{days} d {h} h {m} min"),
    }
}

/// KiB → "1,9 GiB" / "512 MiB".
pub fn format_kib(kib: u64) -> String {
    let mib = kib as f64 / 1024.0;
    if mib >= 1024.0 {
        format!("{:.1} GiB", mib / 1024.0).replace('.', ",")
    } else {
        format!("{mib:.0} MiB")
    }
}

/// Interface de rede com IPv4 (saída de `ip -j -4 addr`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetIface {
    pub name: String,
    pub state: String,
    pub ipv4: Vec<String>,
}

/// Parser mínimo do JSON de `ip -j -4 addr` / `ip -j route`, sem depender de
/// serde_json: extrai pares "chave":"valor" de objetos simples.
fn json_str_field(obj: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\":\"");
    let start = obj.find(&pat)? + pat.len();
    let end = obj[start..].find('"')? + start;
    Some(obj[start..end].to_string())
}

pub fn parse_ip_addr_json(json: &str) -> Vec<NetIface> {
    // Cada interface começa em {"ifindex":...
    json.split("{\"ifindex\"")
        .skip(1)
        .filter_map(|chunk| {
            let name = json_str_field(chunk, "ifname")?;
            let state = json_str_field(chunk, "operstate").unwrap_or_else(|| "UNKNOWN".into());
            let mut ipv4 = Vec::new();
            for part in chunk.split("\"local\":\"").skip(1) {
                if let Some(end) = part.find('"') {
                    let addr = &part[..end];
                    let plen = part
                        .split("\"prefixlen\":")
                        .nth(1)
                        .and_then(|p| p.split(|c: char| !c.is_ascii_digit()).next())
                        .unwrap_or("");
                    ipv4.push(if plen.is_empty() {
                        addr.to_string()
                    } else {
                        format!("{addr}/{plen}")
                    });
                }
            }
            Some(NetIface { name, state, ipv4 })
        })
        .collect()
}

pub fn parse_default_gateway(route_json: &str) -> Option<(String, String)> {
    let obj = route_json
        .split('{')
        .find(|o| o.contains("\"dst\":\"default\""))?;
    Some((
        json_str_field(obj, "gateway")?,
        json_str_field(obj, "dev").unwrap_or_default(),
    ))
}

/// Servidores DNS de um resolv.conf.
pub fn parse_resolv_conf(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix("nameserver"))
        .map(|s| s.trim().to_string())
        .collect()
}

// ------------------------------------------------------------------ coleta

/// Informações do sistema. Campos ausentes ficam `None` — nunca inventados.
#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub os_name: Option<String>,
    pub os_pretty: Option<String>,
    pub os_version: Option<String>,
    pub os_id: Option<String>,
    pub os_variant: Option<String>,
    pub build_id: Option<String>,
    pub base: Option<String>,
    pub kernel_release: Option<String>,
    pub kernel_version: Option<String>,
    pub arch: String,
    pub hostname: Option<String>,
    pub desktop: Option<String>,
    pub session_type: Option<String>,
    pub cpu_model: Option<String>,
    pub cpu_count: Option<usize>,
    pub mem_total_kib: Option<u64>,
    pub mem_available_kib: Option<u64>,
    pub uptime: Option<Duration>,
    pub interfaces: Vec<NetIface>,
    pub gateway: Option<(String, String)>,
    pub dns: Vec<String>,
    pub build_info: BTreeMap<String, String>,
}

fn read(p: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(p).ok()
}

fn read_trim(p: impl AsRef<Path>) -> Option<String> {
    read(p)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn cmd(prog: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(prog).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Caminho do registro de build gravado na imagem.
pub const BUILD_INFO_PATH: &str = "/usr/lib/osmarcio/build-info";

impl SystemInfo {
    pub fn collect() -> Self {
        let osr = read("/etc/os-release")
            .or_else(|| read("/usr/lib/os-release"))
            .map(|t| parse_os_release(&t))
            .unwrap_or_default();
        let (cpu_model, cpu_count) = read("/proc/cpuinfo")
            .map(|t| parse_cpuinfo(&t))
            .map(|(m, c)| (Some(m), Some(c)))
            .unwrap_or((None, None));
        let (mt, ma) = read("/proc/meminfo")
            .map(|t| parse_meminfo(&t))
            .unwrap_or((0, 0));
        let base = read_trim("/etc/debian_version").map(|v| format!("Debian GNU/Linux {v}"));
        let build_info = read(BUILD_INFO_PATH)
            .map(|t| parse_os_release(&t))
            .unwrap_or_default();
        let env = |k: &str| std::env::var(k).ok().filter(|s| !s.is_empty());

        SystemInfo {
            os_name: osr.get("NAME").cloned(),
            os_pretty: osr.get("PRETTY_NAME").cloned(),
            os_version: osr.get("VERSION_ID").cloned(),
            os_id: osr.get("ID").cloned(),
            os_variant: osr.get("VARIANT").cloned(),
            build_id: osr.get("BUILD_ID").cloned(),
            base,
            kernel_release: read_trim("/proc/sys/kernel/osrelease"),
            kernel_version: read_trim("/proc/sys/kernel/version"),
            arch: read_trim("/proc/sys/kernel/arch")
                .unwrap_or_else(|| std::env::consts::ARCH.to_string()),
            hostname: read_trim("/proc/sys/kernel/hostname"),
            desktop: env("XDG_CURRENT_DESKTOP"),
            session_type: env("XDG_SESSION_TYPE"),
            cpu_model,
            cpu_count,
            mem_total_kib: (mt > 0).then_some(mt),
            mem_available_kib: (ma > 0).then_some(ma),
            uptime: read("/proc/uptime").and_then(|t| parse_uptime(&t)),
            interfaces: cmd("ip", &["-j", "-4", "addr"])
                .map(|j| parse_ip_addr_json(&j))
                .unwrap_or_default(),
            gateway: cmd("ip", &["-j", "route", "show", "default"])
                .and_then(|j| parse_default_gateway(&j)),
            dns: read("/run/systemd/resolve/resolv.conf")
                .or_else(|| read("/etc/resolv.conf"))
                .map(|t| parse_resolv_conf(&t))
                .unwrap_or_default(),
            build_info,
        }
    }

    /// Resumo de rede em uma linha, ex.: "conectada — enp0s1 10.0.2.15/24 via 10.0.2.2".
    pub fn network_summary(&self) -> String {
        let up: Vec<_> = self
            .interfaces
            .iter()
            .filter(|i| i.name != "lo" && !i.ipv4.is_empty())
            .collect();
        match (up.first(), &self.gateway) {
            (Some(i), Some((gw, _))) => {
                format!("conectada — {} {} via {}", i.name, i.ipv4.join(", "), gw)
            }
            (Some(i), None) => format!("sem rota padrão — {} {}", i.name, i.ipv4.join(", ")),
            (None, _) => "desconectada (nenhum IPv4 fora do loopback)".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OS_RELEASE: &str = r#"PRETTY_NAME="OSMarcio Linux 0.2.0 (base Debian 13 trixie)"
NAME="OSMarcio Linux"
VERSION_ID="0.2.0"
ID=osmarcio-linux
ID_LIKE=debian
# comentário
BUILD_ID=20261113T120000Z
"#;

    #[test]
    fn os_release_parsing() {
        let m = parse_os_release(OS_RELEASE);
        assert_eq!(m["NAME"], "OSMarcio Linux");
        assert_eq!(m["ID"], "osmarcio-linux");
        assert_eq!(m["VERSION_ID"], "0.2.0");
        assert_eq!(m["BUILD_ID"], "20261113T120000Z");
        assert!(!m.contains_key("# comentário"));
    }

    #[test]
    fn cpuinfo_x86_and_arm() {
        let x86 = "processor\t: 0\nmodel name\t: Intel(R) Core(TM) i7\nprocessor\t: 1\nmodel name\t: Intel(R) Core(TM) i7\n";
        assert_eq!(parse_cpuinfo(x86), ("Intel(R) Core(TM) i7".into(), 2));
        let arm = "processor\t: 0\nBogoMIPS\t: 48.00\nCPU implementer\t: 0x61\nCPU part\t: 0x039\n\nprocessor\t: 1\n";
        let (m, n) = parse_cpuinfo(arm);
        assert!(m.starts_with("Apple"), "{m}");
        assert_eq!(n, 2);
        let rk = "processor\t: 0\nCPU implementer\t: 0x41\nCPU part\t: 0xd08\n";
        assert_eq!(parse_cpuinfo(rk).0, "ARM Cortex-A72");
    }

    #[test]
    fn meminfo_and_uptime() {
        let mi = "MemTotal:        2014312 kB\nMemFree:  100 kB\nMemAvailable:    1800000 kB\n";
        assert_eq!(parse_meminfo(mi), (2014312, 1800000));
        assert_eq!(
            parse_uptime("3725.51 7000.00\n"),
            Some(Duration::from_secs_f64(3725.51))
        );
        assert_eq!(format_duration(Duration::from_secs(3725)), "1 h 2 min");
        assert_eq!(
            format_duration(Duration::from_secs(90_000)),
            "1 d 1 h 0 min"
        );
        assert_eq!(format_kib(2014312), "1,9 GiB");
        assert_eq!(format_kib(524288), "512 MiB");
    }

    #[test]
    fn ip_json_parsing() {
        let addr = r#"[{"ifindex":1,"ifname":"lo","flags":["LOOPBACK","UP"],"mtu":65536,"operstate":"UNKNOWN","addr_info":[{"family":"inet","local":"127.0.0.1","prefixlen":8,"scope":"host"}]},{"ifindex":2,"ifname":"enp0s1","flags":["UP"],"operstate":"UP","addr_info":[{"family":"inet","local":"10.0.2.15","prefixlen":24,"scope":"global","dynamic":true}]}]"#;
        let v = parse_ip_addr_json(addr);
        assert_eq!(v.len(), 2);
        assert_eq!(
            v[1],
            NetIface {
                name: "enp0s1".into(),
                state: "UP".into(),
                ipv4: vec!["10.0.2.15/24".into()]
            }
        );
        let route = r#"[{"dst":"default","gateway":"10.0.2.2","dev":"enp0s1","protocol":"dhcp","flags":[]}]"#;
        assert_eq!(
            parse_default_gateway(route),
            Some(("10.0.2.2".into(), "enp0s1".into()))
        );
        assert_eq!(parse_default_gateway("[]"), None);
    }

    #[test]
    fn network_summary_states() {
        let mut s = SystemInfo::default();
        assert!(s.network_summary().starts_with("desconectada"));
        s.interfaces = vec![NetIface {
            name: "enp0s1".into(),
            state: "UP".into(),
            ipv4: vec!["10.0.2.15/24".into()],
        }];
        assert!(s.network_summary().starts_with("sem rota"));
        s.gateway = Some(("10.0.2.2".into(), "enp0s1".into()));
        assert_eq!(
            s.network_summary(),
            "conectada — enp0s1 10.0.2.15/24 via 10.0.2.2"
        );
    }

    #[test]
    fn resolv_conf() {
        assert_eq!(
            parse_resolv_conf("# x\nnameserver 10.0.2.3\nnameserver ::1\nsearch lan\n"),
            vec!["10.0.2.3", "::1"]
        );
    }
}
