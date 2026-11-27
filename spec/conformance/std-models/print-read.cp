// expect: ok, output "7 x\nok"
// input: "3 5\nhello world\n7\n"
// model: print
// model: read
// model: read_line
// model: rand
// `read` and `read_line` are checked against the bytes the input holds (`3 5`, then the next
// line, then `7`) and `print` writes the values; seeded `rand(lo, hi)` draws are checked
// against their range, and a one-value range against its one value.
let n = read<Int>()
let k = read<Int>()
var bad = if n == 3 and k == 5: "" else: "read"
if bad == "" and read_line() != "hello world": bad = "read_line"
let m = read<Int>()
for _ in 0..200:
    let lo = rand(-5, 5)
    let hi = lo + rand(0, 3)
    let x = rand(lo, hi)
    if bad == "" and (x < lo or x > hi): bad = "rand"
if bad == "" and rand(4, 4) != 4: bad = "rand of one value"
print(m, "x")
print(if bad == "": "ok" else: bad)
