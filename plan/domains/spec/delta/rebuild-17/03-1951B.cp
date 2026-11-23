// 1951B Battle Cows: try no swap, swap with the front, swap with the first stronger cow.
fn wins(a: List<Int>, p: Int) -> Int:
    for i in 0..p:
        if a[i] > a[p]: return 0
    var w = if p > 0: 1 else: 0
    var i = p + 1
    while i < a.len and a[i] < a[p]:
        w += 1
        i += 1
    return w

fn swapped(a: List<Int>, i: Int, j: Int) -> List<Int>:
    var b = a.clone()
    let t = b[i]
    b[i] = b[j]
    b[j] = t
    return b

@!multitest
let n: 2..=1e5 = read()
let k: 1..=n = read()
let a = List(read<[n]1..=1e9>())
let q = k - 1
var ans = max(wins(a, q), wins(swapped(a, 0, q), 0))
if let j = (0..q).find(i => a[i] > a[q]):
    ans max= wins(swapped(a, j, q), j)
print(ans)
