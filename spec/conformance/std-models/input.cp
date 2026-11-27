// expect: ok, output "ok"
// input: "5 400\n"
// model: Input
// engines: next
// `Input.of(s)` and `read<T>(from: s)` against a brute force over seeded token strings: a count
// and that many `Int`s split by random blanks, sometimes with the count one too high (a missing
// token), a token left over, or a token that is no `Int`. A whole read is the list or `None`;
// an `Input` read part by part gives each token in turn. Reads `seed n`; prints the first that
// differs.
let seed0 = read<Int>()
let n = read<Int>()
var r = seed0 % 2147483648

/// The next seeded draw in `lo..=hi`.
fn draw(mut s: Int, lo: Int, hi: Int) -> Int:
    s = (s * 1103515245 + 12345) % 2147483648
    lo + s % (hi - lo + 1)

let blanks = [" ", "\n", "\t", "  ", "\r\n"]
var bad = ""
for k in 0..n:
    let m = draw(mut r, 0, 5)
    let fault = draw(mut r, 0, 3)
    var xs: List<Int> = List.new()
    var s = ""
    if draw(mut r, 0, 1) == 0: s.append(blanks[draw(mut r, 0, 4)])
    s.append((if fault == 1 or fault == 3: m + 1 else: m).show())
    for i in 0..m:
        let v = draw(mut r, -1000000, 1000000)
        xs.push(v)
        s.append(blanks[draw(mut r, 0, 4)])
        s.append(v.show())
    if fault == 2: s.append(" 7")
    if fault == 3: s.append(" x")
    if draw(mut r, 0, 1) == 0: s.append("\n")
    let want = if fault == 0: Some(xs.clone()) else: None
    if read<List<Int>>(from: s) != want: bad = "case {k}: read<List<Int>>(from: {s.show()})"
    var input = Input.of(s)
    if input.read<Int>() != (if fault == 1 or fault == 3: m + 1 else: m):
        bad = "case {k}: the count of {s.show()}"
    for i in 0..m:
        if input.read<Int>() != xs[i]: bad = "case {k}: token {i + 1} of {s.show()}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
