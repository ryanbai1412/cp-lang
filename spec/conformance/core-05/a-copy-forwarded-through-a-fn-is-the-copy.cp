// expect: ok, output "5 1 2 3 9 1 2 3 7 [1, 2, 3]"
// rule: 05.2.2
// A fn that only hands its param to `copy` (std's `clone` is one) gives the same owned copy through one or two forwarders, and the source is left alone.
fn dup(xs: List<Int>) -> List<Int> = copy(xs)
fn dup2(xs: List<Int>) -> List<Int> = dup(xs)
let a = [1, 2, 3]
var b = a.clone()
b.push(9)
var c = dup2(a)
c.push(7)
print(b.len + 2, b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3], a)
