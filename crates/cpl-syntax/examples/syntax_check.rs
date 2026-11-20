//! Prints the syntax diagnostics for each file given: `cargo run -p cpl-syntax --example syntax_check -- a.cp`.

fn main() {
    for path in std::env::args().skip(1) {
        let src = std::fs::read_to_string(&path).expect("readable file");
        let file = cpl_syntax::parse_file(&src);
        print!("{}", cpl_diag::render_all(&file.diagnostics, &path, &src));
        println!(
            "{path}: {} items, {} syntax diagnostics",
            file.items.len(),
            file.diagnostics.len()
        );
    }
}
