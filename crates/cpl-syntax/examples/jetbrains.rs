//! Writes the JetBrains plugin's Kotlin sources generated from the lexer's rules into the
//! directory given, as the plugin's build does: `cargo run -p cpl-syntax --example jetbrains -- DIR`.

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("an output directory"));
    let package = dir.join("dev/cplang/ide");
    std::fs::create_dir_all(&package).expect("the output directory can be made");
    for name in cpl_syntax::grammar::JETBRAINS_KOTLIN {
        let text = cpl_syntax::grammar::jetbrains_kotlin(name).expect("a generated source");
        std::fs::write(package.join(name), text).expect("the source can be written");
    }
}
