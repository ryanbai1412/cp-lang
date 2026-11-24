//! A struct generic over its op (`M: Op<T> = Monoid<T>`): a defaulted type
//! parameter, and `M.Fold` read in its methods, monomorphise per op.

use crate::tests::cpp;

const ACC: &str = concat!(
    "use std.algebra.{Monoid, Op}\n",
    "struct Acc<T, M: Op<T> = Monoid<T>>(m: Monoid<M.Fold>, op: M):\n",
    "    total: M.Fold\n",
    "    fn new<N: Op<T>>(op: N) -> Acc<T, N>:\n",
    "        let m = op.folds()\n",
    "        Acc<T, N>(m, op, m.id)\n",
    "    fn add(mut self, x: T):\n",
    "        total = m.op(total, op.lift(x))\n",
    "    fn sum(self) -> M.Fold = total\n",
);

#[test]
fn a_defaulted_op_param_names_the_monoid_instance() {
    let cpp = cpp(&format!(
        "{ACC}var s: Acc<Int> = Acc(Monoid(0, (a: Int, b: Int) => a + b))\n\
         s.add(read())\n\
         print(s.sum())\n"
    ));
    assert!(cpp.contains("struct Acc_Int_"), "{cpp}");
    assert!(!cpp.contains("Acc_Int_Monoid"), "{cpp}");
    assert!(cpp.contains("inline ll Acc_Int_"), "{cpp}");
    assert!(!cpp.contains("cpl::OptInt"), "{cpp}");
}

#[test]
fn m_fold_is_t_for_a_monoid_and_optional_for_a_closure() {
    let cpp = cpp(&format!(
        "{ACC}var s = Acc(Monoid(0, (a: Int, b: Int) => a + b))\n\
         var lo = Acc((a: Int, b: Int) => if a < b: a else: b)\n\
         let x: Int = read()\n\
         s.add(x)\n\
         lo.add(x)\n\
         print(s.sum(), lo.sum() ?? -1)\n"
    ));
    assert!(cpp.contains("inline ll Acc_Int_"), "{cpp}");
    assert!(
        cpp.contains("cpl::OptInt& Acc_Int_Fn_Int_Int_Int_"),
        "{cpp}"
    );
}

#[test]
fn a_closure_segtree_folds_to_an_optional_and_a_sum_tree_does_not() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "domain V = 0..n\n",
        "let a: [V]Int = [V] v => Int(v) % 5\n",
        "let low = SegTree((x: Int, y: Int) => if x <= y: x else: y, a)\n",
        "let sum = SegTree(+, a)\n",
        "let s: Int = sum.all\n",
        "print(low.all ?? -1, s)\n",
    ));
    assert!(
        cpp.contains("struct SegTree_D0_V_Int_Int_Fn_Int_Int_Int_"),
        "{cpp}"
    );
    assert!(cpp.contains("std::vector<cpl::OptInt> t;"), "{cpp}");
    assert!(cpp.contains("std::vector<ll> t;"), "{cpp}");
}

#[test]
fn a_closure_segtree2d_folds_rectangles_to_an_optional() {
    let cpp = cpp(concat!(
        "use std.segtree.SegTree2D\n",
        "var g = SegTree2D<0..2, 0..3, Int>((a: Int, b: Int) => if a < b: a else: b, fill: 5)\n",
        "g.set(1, 2, read())\n",
        "print(g.fold(0..1, 0..3) ?? -1, g.all ?? -1)\n",
    ));
    assert!(cpp.contains("cpl::OptInt SegTree2D_"), "{cpp}");
}
