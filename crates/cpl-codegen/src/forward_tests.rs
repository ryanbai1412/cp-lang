//! A call to a fn that returns one small expression of its params is that
//! expression (`crate::forward`), unless an argument could do something.

use crate::tests::cpp;

/// The C++ past the source listing at the top of the file.
fn program(source: &str) -> String {
    let cpp = cpp(source);
    cpp[cpp.find("namespace prog").expect("the program")..].to_string()
}

#[test]
fn a_call_to_a_fn_that_returns_its_param_is_that_argument() {
    let cpp = program(concat!(
        "struct Lift(k: Int):\n",
        "    fn lift(self, x: Int) -> Int = x\n",
        "let n: 1..=10 = read()\n",
        "let l = Lift(3)\n",
        "print(l.lift(n) + l.k)\n",
    ));
    assert!(!cpp.contains("_lift("), "{cpp}");
}

#[test]
fn an_other_argument_that_does_something_still_runs() {
    let cpp = program(concat!(
        "fn second(a: Int, b: Int) -> Int = b\n",
        "fn noisy(x: Int) -> Int:\n",
        "    print(x)\n",
        "    x\n",
        "let n: 1..=10 = read()\n",
        "print(second(noisy(n), n + 1))\n",
    ));
    assert!(cpp.contains("second("), "{cpp}");
    assert!(cpp.contains("noisy("), "{cpp}");
}

#[test]
fn a_pure_other_argument_is_dropped_with_the_call() {
    let cpp = program(concat!(
        "fn same(a: Int, b: Int) -> Int = a\n",
        "let n: 1..=10 = read()\n",
        "print(same(n, n * 2))\n",
    ));
    assert!(!cpp.contains("same("), "{cpp}");
}

#[test]
fn a_call_to_a_fn_that_returns_an_expression_of_its_params_is_that_expression() {
    let cpp = program(concat!(
        "fn near(a: Int, b: Int) -> Bool = a < b\n",
        "let n: 1..=10 = read()\n",
        "let m: 1..=10 = read()\n",
        "print(yes(near(n, m)))\n",
    ));
    assert!(!cpp.contains("near("), "{cpp}");
}

#[test]
fn an_argument_the_expression_moves_still_runs_once_in_a_call() {
    let cpp = program(concat!(
        "fn near(a: Int, b: Int) -> Bool = a < b\n",
        "fn noisy(x: Int) -> Int:\n",
        "    print(x)\n",
        "    x\n",
        "let n: 1..=10 = read()\n",
        "print(yes(near(noisy(n), n)))\n",
    ));
    assert!(cpp.contains("near("), "{cpp}");
}

#[test]
fn a_param_read_twice_keeps_its_call() {
    let cpp = program(concat!(
        "fn twice(a: Int) -> Int = a + a\n",
        "let n: 1..=10 = read()\n",
        "print(twice(n * 3))\n",
    ));
    assert!(cpp.contains("twice("), "{cpp}");
}

#[test]
fn a_ranged_param_keeps_the_call_that_checks_it() {
    let cpp = program(concat!(
        "fn low(a: 0..=5) -> Int = a\n",
        "let n: 1..=10 = read()\n",
        "print(low(n))\n",
    ));
    assert!(cpp.contains("low("), "{cpp}");
}
