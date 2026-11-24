// expect: ok, output "15.0"
// rule: 13.3.6
// The cold tail of a recursive fn sorts by a key closure: the key's parameter belongs to the sort,
// not to the tail's inputs, so the tail builds and answers as it would inline.
fn go(n: Int) -> F64:
    if n == 0: return 0.0
    var xs: List<F64> = []
    for i in 0..n: xs.push(F64(i) + go(n - 1))
    let best = xs.sorted(by: x => desc(x)).take(2)
    best.sum()
print("{go(4):.1}")
