//! A fn handed a table literal reads each cell as it needs it (`crate::table`).

use crate::tests::cpp;

#[test]
fn a_prefix_of_a_table_literal_builds_no_table() {
    let cpp = cpp(concat!(
        "let h: 1..=3000 = read()\n",
        "let w: 1..=3000 = read()\n",
        "domain Row = 0..h\n",
        "domain Col = 0..w\n",
        "let rank: [Row, Col]Int = [Row, Col] (i, j) => read()\n",
        "fn low(x: Int) -> Int:\n",
        "    let p = Prefix([Row, Col] (i, j) => Int(rank[i, j] <= x))\n",
        "    p.fold(.., ..)\n",
        "print(low(5))\n",
    ));
    assert!(cpp.contains("cpl_tab_"), "{cpp}");
    let low = &cpp[cpp.find("ll low(").unwrap()..];
    let low = &low[..low.find("\n}").unwrap()];
    assert!(
        !low.contains("zero_grid"),
        "no table for the literal:\n{low}"
    );
    let tab = &cpp[cpp.find("(CplT&& ").unwrap() + 8..];
    let cell = &tab[..tab.find(')').unwrap()];
    let tab = &tab[..tab.find("\n}").unwrap()];
    assert!(
        !tab.contains(&format!("{cell}.d.")),
        "the cell callback is not walked as a grid:\n{tab}"
    );
}

#[test]
fn a_cell_that_may_trap_keeps_the_table() {
    let cpp = cpp(concat!(
        "let h: 1..=3000 = read()\n",
        "domain Row = 0..h\n",
        "domain Col = 0..3\n",
        "let d: Int = read()\n",
        "let p = Prefix([Row, Col] (i, j) => 100 / (Int(i) - d))\n",
        "print(p.fold(.., ..))\n",
    ));
    assert!(!cpp.contains("cpl_tab_"), "{cpp}");
}
