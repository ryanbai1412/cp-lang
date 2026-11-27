// expect: ok, output "1 7 12"
// rule: R06.1.6
// Fns of one name whose required labels differ are overloads: a call takes the one whose labels it gives.
fn f() -> Int = 1
fn f(; at: Int) -> Int = at + 6
fn f(; lo: Int, hi: Int = 2) -> Int = lo + hi
print(f(), f(at: 1), f(lo: 10))
