// expect: ok, output "ok"
// input: "7 400\n"
// model: reversed
// model: reverse
// `reversed` and in-place `reverse`, on whole lists, slices and `rev` walks, against an index
// loop: seeded random pushes, pops and reversals, compared after each op. Reads `seed n`;
// prints the first op that differs.
fn brute(a: List<Int>, lo: Int, hi: Int) -> List<Int>:
    var b = a.clone()
    for i in lo..hi: b[i] = a[hi - 1 - (i - lo)]
    b

let seed0 = read<Int>()
let n = read<Int>()
var seed = seed0
var a: List<Int> = []
var bad = ""
for k in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let op = seed % 5
    let x = seed / 5 % 1000
    seed = (seed * 1103515245 + 12345) % 2147483648
    let lo = if a.len == 0: 0 else: seed % (a.len + 1)
    let hi = lo + (seed / 4096) % (a.len - lo + 1)
    if op <= 1 or a.len == 0:
        a.push(x)
    elif op == 2:
        a.pop()
    elif op == 3:
        let want = brute(a, lo, hi)
        a[lo..hi].reverse()
        if a != want and bad == "": bad = "reverse {k}"
    else:
        let want = brute(a, 0, a.len)
        if a.reversed() != want and bad == "": bad = "reversed {k}"
        if a[lo..hi].reversed() != brute(a, lo, hi)[lo..hi] and bad == "": bad = "slice {k}"
        if a.rev.collect() != want and bad == "": bad = "rev {k}"
print(if bad == "": "ok" else: bad)
