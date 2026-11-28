// expect: ok, output "ok"
// input: "1 300\n"
// model: adjacent
// model: pairs
// model: with_prev
// model: with_next
// model: cyclic_pairs
// model: zip
// model: zip_exact
// model: windows
// model: chunks
// model: chunks_padded
// model: take_while
// model: drop_while
// model: filter_map
// model: flat_map
// model: flatten
// model: present
// model: all_some
// model: chain
// model: product
// model: iterate
// model: index_pairs
// model: split
// model: runs
// The std generators over Lists against index loops: each op draws two short random lists
// and checks every pairing, window, chunk, prefix cut, flattening, chaining, split and run.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.seq.{zip_exact}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let len = rng.int(0, 7)
    let a = [rng.int(-2, 2) for _ in 0..len]
    let b = [rng.int(0, 3) for _ in 0..rng.int(0, 5)]
    let w = rng.int(1, 8)
    let t = rng.int(-2, 2)
    let opt = [if x > t: Some(x) else: None for x in a]
    var tw = 0
    while tw < len and a[tw] < t: tw += 1
    var cuts = [-1] + [j for j in 0..len if a[j] == t] + [len]
    var rs: List<List<Int>> = []
    for x in a:
        if rs.len > 0 and rs.last!.last! == x: rs[rs.len - 1].push(x)
        else: rs.push([x])
    var ls: List<List<Int>> = []
    for x in a:
        if ls.len > 0 and ls.last!.last! + 1 == x: ls[ls.len - 1].push(x)
        else: ls.push([x])
    let zl = if len < b.len: len else: b.len
    var why = ""
    if a.adjacent.collect() != [(a[j], a[j + 1]) for j in 0..len - 1]: why = "adjacent"
    elif a.pairs().collect() != [(a[i], a[j]) for i in 0..len, j in i + 1..len]: why = "pairs"
    elif a.with_prev.collect() != [(if j == 0: None else: Some(a[j - 1]), a[j]) for j in 0..len]: why = "with_prev"
    elif a.with_next().collect() != [(a[j], if j + 1 == len: None else: Some(a[j + 1])) for j in 0..len]: why = "with_next"
    elif a.cyclic_pairs().collect() != [(a[j], a[(j + 1) % len]) for j in 0..len]: why = "cyclic_pairs"
    elif zip(a, b).collect() != [(a[j], b[j]) for j in 0..zl]: why = "zip"
    elif zip_exact(a, a.rev).collect() != [(a[j], a[len - 1 - j]) for j in 0..len]: why = "zip_exact"
    elif [x.collect() for x in a.windows(w)] != [[a[j] for j in i..i + w] for i in 0..len - w + 1]: why = "windows"
    elif [x.keys for x in a.windows(w)] != [i..i + w for i in 0..len - w + 1]: why = "window keys"
    elif [x.collect() for x in a.chunks(w)] != [[a[j] for j in i * w..(if (i + 1) * w < len: (i + 1) * w else: len)] for i in 0..(len + w - 1) / w]: why = "chunks"
    elif a.chunks_padded(w, 9).collect() != [[if j < len: a[j] else: 9 for j in i * w..(i + 1) * w] for i in 0..(len + w - 1) / w]: why = "chunks_padded"
    elif a.take_while(x => x < t).collect() != [a[j] for j in 0..tw]: why = "take_while"
    elif a.drop_while(x => x < t).collect() != [a[j] for j in tw..len]: why = "drop_while"
    elif a.filter_map(x => if x > t: Some(x * 5) else: None).collect() != [x * 5 for x in a if x > t]: why = "filter_map"
    elif b.flat_map(x => 0..x).collect() != [j for x in b, j in 0..x]: why = "flat_map"
    elif [a, b, a].flatten().collect() != a + b + a: why = "flatten"
    elif opt.present().collect() != [x for x in a if x > t]: why = "present"
    elif opt.all_some() != (if a.all(x => x > t): Some(a) else: None): why = "all_some"
    elif chain(a, b).collect() != a + b or chain(b, a, b).collect() != b + a + b: why = "chain"
    elif product(a, b).collect() != [(x, y) for x in a, y in b]: why = "product"
    elif iterate(if len > 0: Some(0) else: None, j => if j + 1 < len: Some(j + 1) else: None).collect() != [j for j in 0..len]: why = "iterate"
    elif (0..len).index_pairs().collect() != [(i, j) for i in 0..len, j in i + 1..len]: why = "index_pairs"
    elif [p.collect() for p in a.split(t)] != [[a[j] for j in cuts[i] + 1..cuts[i + 1]] for i in 0..cuts.len - 1]: why = "split"
    elif [p.collect() for p in a.split(x => x == t)] != [[a[j] for j in cuts[i] + 1..cuts[i + 1]] for i in 0..cuts.len - 1]: why = "split by"
    elif [r.collect() for r in a.runs()] != rs: why = "runs"
    elif [r.collect() for r in a.runs(by: x => x * x)] != [[a[j] for j in r.keys] for r in a.runs(link: (x, y) => x * x == y * y)]: why = "runs by"
    elif [r.collect() for r in a.runs(link: (x, y) => x + 1 == y)] != ls: why = "runs link"
    if why != "":
        bad = "op {k}: {why} on {a} and {b} (w {w}, t {t})"
        break
print(if bad == "": "ok" else: bad)
