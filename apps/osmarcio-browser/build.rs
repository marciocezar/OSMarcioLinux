// O libcef.so é instalado no mesmo diretório do executável
// (/usr/lib/osmarcio-browser/). RPATH=$ORIGIN faz o carregador dinâmico
// procurá-lo ali, sem LD_LIBRARY_PATH.
//
// Também exporta OSMB_CEF_RS_VERSION: a versão REAL do crate `cef` (bindings
// Rust) resolvida no Cargo.lock, mostrada na tela Sobre e em --version.
fn main() {
    println!("cargo::rustc-link-arg-bins=-Wl,-rpath,$ORIGIN");
    let lock = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock");
    println!("cargo::rerun-if-changed={lock}");
    let text = std::fs::read_to_string(lock).unwrap_or_default();
    let v = text
        .split("[[package]]")
        .find(|p| p.contains("\nname = \"cef\"\n"))
        .and_then(|p| p.lines().find_map(|l| l.strip_prefix("version = \"")))
        .map(|v| v.trim_end_matches('"').to_string())
        .unwrap_or_else(|| "Não disponível".into());
    println!("cargo::rustc-env=OSMB_CEF_RS_VERSION={v}");
}
