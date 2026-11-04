// Embute o ícone e as informações de versão no .exe do Windows.
fn main() {
    println!("cargo:rerun-if-changed=assets/stardust.rc");
    println!("cargo:rerun-if-changed=assets/stardust.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/stardust.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("falha ao embutir recursos do Windows");
    }
}
