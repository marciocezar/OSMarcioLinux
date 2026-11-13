// O libcef.so é instalado no mesmo diretório do executável
// (/usr/lib/osmarcio-browser/). RPATH=$ORIGIN faz o carregador dinâmico
// procurá-lo ali, sem LD_LIBRARY_PATH.
fn main() {
    println!("cargo::rustc-link-arg-bins=-Wl,-rpath,$ORIGIN");
}
