//! A call to a fn that only returns a param is that argument
//! (`crate::forward`), unless another argument could do something.

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
