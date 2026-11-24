// expect: ok, output "1 2 7 7\n7 7 7 7"
// rule: 05.3.5
// `let old = s.ks` must not be moved out when only one branch sets `s.ks` again before the read.
struct S:
    var ks: List<Int>
fn grow(mut s: S, c: Bool):
    let old = s.ks
    if c: s.ks = [7, 7]
    print(old, s.ks)
var s = S(ks: [1, 2])
grow(mut s, true)
grow(mut s, false)
