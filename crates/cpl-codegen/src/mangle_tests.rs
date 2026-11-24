//! A struct instance's C++ name spells its type args, past a trailing run
//! of params given their defaults.

use crate::tests::cpp;

#[test]
fn a_defaulted_struct_param_given_its_default_is_one_instance_left_out_of_the_name() {
    let cpp = cpp(concat!(
        "struct Pair<T, U = Int>(a: T, b: U)\n",
        "let n: 1..=10 = read()\n",
        "let p = Pair<Int>(n, 1)\n",
        "let q = Pair<Int, Int>(n, 2)\n",
        "let r = Pair<Int, Bool>(n, true)\n",
        "print(p.a + q.b + r.a, yes(r.b))\n",
    ));
    let cpp = &cpp[cpp.find("namespace prog").expect("the program")..];
    assert_eq!(cpp.matches("struct Pair_").count(), 2, "{cpp}");
    assert!(cpp.contains("struct Pair_Int_1 {"), "{cpp}");
    assert!(cpp.contains("struct Pair_Int_Bool_2 {"), "{cpp}");
}
