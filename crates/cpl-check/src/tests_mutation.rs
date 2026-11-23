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
