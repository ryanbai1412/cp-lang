// A top-level `let side = xs.map(f)` is a view the file's fns read too, per test.
@!multitest
let (n, k) = read<(1..=9, 1..=99)>()
domain P = 1..=n
enum Side: Small(Int) | Big(Int)
let side = read<[P]1..=k>().map(b => if b * b <= k: Small(b) else: Big(k / b))
let twice = read<[P]Int>().map(b => b * 2)
fn weigh(i: P) -> Int = match side[i]:
    Small(b) => b
    Big(q) => -q
fn tail() -> Int = twice.take(2).sum()
print([weigh(i) for i in P], twice[P.first!], tail(), side.len)
