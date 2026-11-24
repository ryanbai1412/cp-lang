//! A named product domain is the tuple it names, so two bounds that give a
//! shared parameter `Cell` and `(R, C)` agree on one `V`.

use cpl_check::Session;

#[test]
fn a_named_product_and_its_tuple_give_one_shared_bound_arg() {
    let mut session = Session::new();
    let src = "domain R = 0..3\ndomain C = 0..3\ndomain Cell = (R, C)\n\
        fn nb(p: Cell) -> yields Cell:\n    for q in p.adj4: yield q\n\
        let ps = bfs(nb, from: [(R(0), C(0))])\n\
        print(ps.get((R(2), C(0))) ?? -1)\n";
    session.set_file("main.cp", src);
    let codes: Vec<_> = (session.diagnostics("main.cp").iter())
        .map(|d| d.code.0)
        .collect();
    assert_eq!(codes, Vec::<&str>::new());
}
