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

#[test]
fn a_void_fn_that_forwards_its_params_to_one_call_is_that_call() {
    let cpp = program(concat!(
        "struct Tally(t: List<Int>):\n",
        "    fn put(mut self, p: Int, x: Int, add: Bool):\n",
        "        if add: t[p] += x else: t[p] = x\n",
        "        print(t[p])\n",
        "    fn set(mut self, p: Int, x: Int) = self.put(p, x, false)\n",
        "var s = Tally([0, 0, 0])\n",
        "let n: 0..3 = read()\n",
        "s.set(n, 5)\n",
        "s.set(n, 7)\n",
    ));
    assert!(!cpp.contains("_set("), "{cpp}");
    // No bigger than the program that calls `put` itself.
    let direct = program(concat!(
        "struct Tally(t: List<Int>):\n",
        "    fn put(mut self, p: Int, x: Int, add: Bool):\n",
        "        if add: t[p] += x else: t[p] = x\n",
        "        print(t[p])\n",
        "var s = Tally([0, 0, 0])\n",
        "let n: 0..3 = read()\n",
        "s.put(n, 5, false)\n",
        "s.put(n, 7, false)\n",
    ));
    assert_eq!(cpp, direct);
}

#[test]
fn a_forwarder_that_reorders_its_params_keeps_its_call() {
    let cpp = program(concat!(
        "fn show(a: Int, b: Int):\n",
        "    print(a - b)\n",
        "fn swapped(a: Int, b: Int) = show(b, a)\n",
        "let n: 1..=10 = read()\n",
        "swapped(n, n + 1)\n",
    ));
    assert!(cpp.contains("swapped("), "{cpp}");
}

#[test]
fn a_forwarder_whose_added_args_would_grow_its_many_sites_keeps_its_call() {
    let cpp = program(concat!(
        "fn among(g: Int, lo: Int, hi: Int) -> Int = g * (hi - lo)\n",
        "let n: Int = read()\n",
        "fn size(g: Int) -> Int = among(g, 0, n)\n",
        "let x: Int = read()\n",
        "print(size(x), size(x + 1), size(x + 2), size(x + 3))\n",
    ));
    assert!(cpp.contains("size("), "{cpp}");
}
