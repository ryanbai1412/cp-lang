// expect: ok, output "1 1\n7 1\n1 2\n1 1\n2 3 1\nYES"
// rule: 02.1.2
// input: "1\n"
// A local whose last use is the value handed out moves into it: a struct, a
// tuple or `Some` of a tuple, returned early or as the tail, and the tuple an
// inlined call gives back.
struct S(xs: List<Int>, k: Int)
fn tail_some(n: Int) -> ?(Int, List<Int>):
    var x = [n]
    x.push(1)
    Some((n, x))
fn early_some(n: Int) -> ?(Int, List<Int>):
    var x = [n]
    x.push(1)
    if n > 5: return Some((n, x))
    None
fn tail_struct(n: Int) -> S:
    var xs = [n]
    xs.push(2)
    S(xs, n)
fn tail_tuple(n: Int) -> (List<Int>, List<Int>):
    var p = [n]
    var q = [n, n]
    p.push(1)
    (p, q)
let n: Int = read()
print(tail_some(n)!.1)
print(early_some(n + 6)!.1)
print(tail_struct(n).xs)
print(tail_tuple(n).0)
let (y, z) = [1, 2, 3].partition(v => v > n)
print(y, z)
print(yes([2, 4].partition(x => x % 2 == 0) == ([2, 4], [])))
