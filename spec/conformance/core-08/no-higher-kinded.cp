// expect: E0501 at 1
// rule: 08.5.3
fn f<F<_>>(x: F<Int>) -> Int = 0
