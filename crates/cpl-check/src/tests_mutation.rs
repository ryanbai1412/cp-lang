//! A `let` can't change, whichever way the change is spelled.

use crate::tests::{check, messages};

#[test]
fn a_mut_self_fn_called_as_a_method_needs_a_receiver_that_can_change() {
    let decl = "fn bump(mut self: List<Int>):\n    self[0] += 1\n";
    assert_eq!(check(&format!("{decl}var a = [1]\na.bump()\n")), vec![]);
    assert_eq!(
        messages(&check(&format!("{decl}let a = [1]\na.bump()\n"))),
        ["`a` is a `let`, so it can't be changed by `bump`"]
    );
    assert_eq!(
        messages(&check("let a = [1, 2]\na.swap_at(0, 1)\n")),
        ["`a` is a `let`, so it can't be changed by `swap_at`"]
    );
}

#[test]
fn each_name_a_tuple_assignment_writes_must_be_a_var() {
    assert_eq!(check("var a = 1\nvar b = 2\n(a, b) = (b, a)\n"), vec![]);
    assert_eq!(
        messages(&check("let a = 1\nvar b = 2\n(a, b) = (b, a)\n")),
        ["`a` is a `let`, so it can't be assigned"]
    );
    assert_eq!(
        messages(&check("let xs = [1, 2]\n(xs[0], xs[1]) = (2, 1)\n")),
        ["`xs` is a `let`, so it can't be assigned"]
    );
}

#[test]
fn a_library_fn_that_changes_an_argument_takes_it_as_mut() {
    assert_eq!(
        check("var a = [3, 1]\nsort(mut a)\nvar x = 1\nvar y = 2\nswap(mut x, mut y)\n"),
        vec![]
    );
    assert_eq!(
        messages(&check("var a = [3, 1]\nsort(a, by: v => -v)\n")),
        ["`sort` changes `a`, so the argument must be passed as `mut a`"]
    );
    assert_eq!(
        messages(&check("var a = [3, 1]\nreverse(a)\n")),
        ["`reverse` changes `a`, so the argument must be passed as `mut a`"]
    );
    assert_eq!(
        messages(&check("var x = 1\nvar y = 2\nswap(x, mut y)\n")),
        ["`swap` changes `x`, so the argument must be passed as `mut x`"]
    );
    // A fn of the program's own named `swap` takes what it declares.
    assert_eq!(
        check("fn swap(a: Int, b: Int) -> Int = a + b\nprint(swap(1, 2))\n"),
        vec![]
    );
}

#[test]
fn a_did_you_mean_keeps_values_and_types_apart() {
    use crate::check::closest;
    assert_eq!(closest("cnt", ["Int", "count"].into_iter()), None);
    assert_eq!(closest("vec", ["Vec"].into_iter()).as_deref(), Some("Vec"));
    assert_eq!(
        closest("Lst", ["List"].into_iter()).as_deref(),
        Some("List")
    );
}
