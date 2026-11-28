// expect: ok, output "ok"
// input: "1 300\n"
// model: len
// len over seeded random lists, strings, ranges and filtered comprehensions, against counting
// the items by hand. Reads `seed n`; prints the first value whose length differs.
let seed0 = read<Int>()
let n = read<Int>()
var seed = seed0
var bad = ""
for k in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let m = seed % 12
    var xs: List<Int> = []
    var s = ""
    for _ in 0..m:
        seed = (seed * 1103515245 + 12345) % 2147483648
        xs.push(seed % 21 - 10)
        s = s + "{Char(97 + seed / 21 % 26)}"
    var counted = 0
    var positive = 0
    for x in xs:
        counted += 1
        if x > 0: positive += 1
    if len(xs) != counted: bad = "op {k}: len of a list of {counted}"
    elif len(s) != counted: bad = "op {k}: len of a string of {counted}"
    elif len(0..m) != m: bad = "op {k}: len of 0..{m}"
    elif len([x for x in xs if x > 0]) != positive: bad = "op {k}: len of {positive} positives"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
