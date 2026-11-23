let n: 1..=100 = read()
let a: [0..n]Int = read()
var by_mod: List<List<Int>> = [[] for _ in 0..3]
for x in a:
    by_mod[x % 3].push(x)
print(by_mod[0].len)
fn diag(pos: Int, val: Int) -> Int: pos - val + n - 1
var bads: [0..2 * n - 1]List<Int> = []
for _ in 0..n:
    let (i, x): (1..=n, 1..=n) = read()
    bads[diag(i, x)].push(i - 1)
print(bads[0].len)
domain V = 1..=n
var children: [V]List<V> = []
for u in V:
    repeat read<Int>():
        let c: V = read()
        children[u].push(c)
print(children[V(1)].len)
