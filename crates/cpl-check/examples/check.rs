//! Checks each file given and prints its diagnostics:
//! `cargo run -p cpl-check --example check -- a.cp`.

fn main() {
    let mut db = cpl_check::Database::new();
    for path in std::env::args().skip(1) {
        let src = std::fs::read_to_string(&path).expect("readable file");
        db.set_file(&path, src.clone());
        let diags = db.diagnostics(&path);
        print!("{}", cpl_diag::render_all(&diags, &path, &src));
    }
}
