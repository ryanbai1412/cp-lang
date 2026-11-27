// expect: ok, output "131988640220562521--m\n16540112210267081-846073319757576596 -792319538577296259 5 960104671314384684-y\n5 mx\n-154490258858266979 -8, -8 -154490258858266979"
// input: "7 4 0\n"
// model: write
// compare: stdout
// engines: next
// `write(..xs, sep:)` against a brute force that builds the same text by hand, over seeded
// random values: `Int`s (sign and digits), `Str` words, `Char`s, lists of each, a list of lists
// and tuples, one to three values, with `sep:` absent or a random separator. Reads `seed n
// side`: side 0 writes through `write`, side 1 through the brute force; the two outputs agree.
let seed0 = read<Int>()
let n = read<Int>()
let side = read<Int>()
var s = seed0 % 2147483648

/// The next seeded draw in `lo..=hi`.
fn draw(mut s: Int, lo: Int, hi: Int) -> Int:
    s = (s * 1103515245 + 12345) % 2147483648
    lo + s % (hi - lo + 1)

fn big(mut s: Int) -> Int:
    let k = draw(mut s, 0, 3)
    if k == 0: return draw(mut s, -9, 9)
    let hi = draw(mut s, 0, 999_999_999)
    let lo = draw(mut s, 0, 999_999_999)
    let x = hi * 1_000_000_000 + lo
    if k == 1: -x else: x

fn word(mut s: Int) -> Str:
    var w = ""
    for _ in 0..draw(mut s, 1, 4): w.push(Char(97 + draw(mut s, 0, 25)))
    w

fn digits(mut out: Str, x: Int):
    if x < 0: out.push('-')
    var m = if x < 0: -x else: x
    var ds: List<Char> = []
    while true:
        ds.push(Char(48 + m % 10))
        m = m / 10
        if m == 0: break
    for i in 0..ds.len: out.push(ds[ds.len - 1 - i])

fn join_ints(mut out: Str, xs: List<Int>, sep: Str):
    for i in 0..xs.len:
        if i > 0: out.append(sep)
        digits(mut out, xs[i])

var text = ""
for _ in 0..n:
    let kind = draw(mut s, 0, 7)
    let has_sep = draw(mut s, 0, 1) == 1
    let sep = [", ", "", "-", "\n"][draw(mut s, 0, 3)]
    let gap = if has_sep: sep else: " "
    let x = big(mut s)
    let y = big(mut s)
    let w = word(mut s)
    let c = Char(97 + draw(mut s, 0, 25))
    let xs = [big(mut s) for _ in 0..draw(mut s, 0, 4)]
    let cs = [Char(97 + draw(mut s, 0, 25)) for _ in 0..draw(mut s, 0, 4)]
    if side == 0:
        if kind == 0:
            if has_sep: write(x, sep: sep) else: write(x)
        elif kind == 1:
            if has_sep: write(x, w, sep: sep) else: write(x, w)
        elif kind == 2:
            if has_sep: write(xs, sep: sep) else: write(xs)
        elif kind == 3:
            if has_sep: write(cs, sep: sep) else: write(cs)
        elif kind == 4:
            if has_sep: write(x, xs, c, sep: sep) else: write(x, xs, c)
        elif kind == 5:
            if has_sep: write([xs, [y]], sep: sep) else: write([xs, [y]])
        elif kind == 6:
            if has_sep: write([(x, y), (y, x)], sep: sep) else: write([(x, y), (y, x)])
        else:
            if has_sep: write(w, c, (x, y), sep: sep) else: write(w, c, (x, y))
        write('\n')
    else:
        if kind == 0:
            digits(mut text, x)
        elif kind == 1:
            digits(mut text, x)
            text.append(gap)
            text.append(w)
        elif kind == 2:
            join_ints(mut text, xs, gap)
        elif kind == 3:
            for i in 0..cs.len:
                if i > 0 and has_sep: text.append(sep)
                text.push(cs[i])
        elif kind == 4:
            digits(mut text, x)
            text.append(gap)
            join_ints(mut text, xs, " ")
            text.append(gap)
            text.push(c)
        elif kind == 5:
            join_ints(mut text, xs, " ")
            text.append(if has_sep: sep else: "\n")
            digits(mut text, y)
        elif kind == 6:
            join_ints(mut text, [x, y], " ")
            text.append(if has_sep: sep else: "\n")
            join_ints(mut text, [y, x], " ")
        else:
            text.append(w)
            text.append(gap)
            text.push(c)
            text.append(gap)
            join_ints(mut text, [x, y], " ")
        text.push('\n')
if side == 1: write(text)
