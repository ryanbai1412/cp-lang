//! Two impls of one trait whose heads match the same types overlap, so the
//! second is E0202; impls with distinct heads stay apart.

use cpl_check::Session;

fn codes(src: &str) -> Vec<&'static str> {
    let mut session = Session::new();
    session.set_file("main.cp", src);
    (session.diagnostics("main.cp").iter())
        .map(|d| d.code.0)
        .collect()
}

#[test]
fn an_impl_stated_twice_for_one_head_is_e0202() {
    let one = "struct W<S>(base: S)\n\
        impl Sized for W<S: Sized>:\n    @get fn len(self) -> Int = self.base.len\n";
    let src = format!(
        "{one}{}print(W([1, 2]).len)\n",
        &one["struct W<S>(base: S)\n".len()..]
    );
    assert!(codes(&src).contains(&"E0202"), "{:?}", codes(&src));
    let once = format!("{one}print(W([1, 2]).len)\n");
    assert_eq!(codes(&once), Vec::<&str>::new());
}
