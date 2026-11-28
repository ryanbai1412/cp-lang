// expect: ok, output "ok"
// input: "1 300\n"
// model: unzip
// unzip over seeded random lists of pairs, and over a lazy view of them, against splitting
// each pair by hand. Reads `seed n`; prints the first list that differs.
let seed0 = read<Int>()
let n = read<Int>()
var seed = seed0
var bad = ""
for k in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let len = seed % 9
    var ps: List<(Int, Char)> = []
    var xs: List<Int> = []
    var cs: List<Char> = []
    for _ in 0..len:
        seed = (seed * 1103515245 + 12345) % 2147483648
        let x = seed % 21 - 10
        let c = Char(97 + seed / 21 % 26)
        ps.push((x, c))
        xs.push(x)
        cs.push(c)
    if ps.unzip() != (xs.clone(), cs.clone()):
        bad = "op {k}: unzip of {len} pairs"
    elif ps.rev.unzip() != (xs.rev.collect(), cs.rev.collect()):
        bad = "op {k}: unzip of the reversed {len} pairs"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
