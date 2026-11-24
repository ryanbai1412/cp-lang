//! The checker re-infers a call's arguments and receivers quietly to pick an
//! overload or bind a generic. A quiet pass leaves nothing behind for the
//! pass that counts.

use cpl_check::Session;

/// Re-inferring a mutable slice's receiver must not leave its lend behind,
/// or the real pass sees two disjoint slices alias (E0602).
#[test]
fn a_quietly_reinferred_slice_receiver_leaves_no_lend() {
    let mut session = Session::new();
    let src = "var xs = [4, 3, 2, 1]\nlet a = xs[0..2]\nlet b = xs[2..4]\na.sort()\nb.sort()\nprint(xs)\n";
    session.set_file("main.cp", src);
    let codes: Vec<_> = (session.diagnostics("main.cp").iter())
        .map(|d| d.code.0)
        .collect();
    assert_eq!(codes, Vec::<&str>::new());
}
