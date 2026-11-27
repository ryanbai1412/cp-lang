// expect: ok, output "10 6 7"
// rule: R05.10.2
// A body that makes two set parameters one set, by holding either fn in one `var`, has its
// callers pass one set for both, so a call joins its two arguments' sets.
fn pick(a: Int -> Int, b: Int -> Int, c: Bool) -> Int -> Int:
    var x = a
    if c: x = b
    x
let k = 2
print(pick(x => x + 1, x => x * k, true)(5), pick(x => x + 1, x => x * 2, false)(5), pick(x => x + k, x => x, false)(5))
