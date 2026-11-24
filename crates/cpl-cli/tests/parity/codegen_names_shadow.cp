// A local may take any name the emitted C++ uses for its own temporaries:
// print blocks, reads, joins, `get` and Range.first all name theirs cpl_*.
let at = 1
let start = 2
let from = 0
let a = [5, 6, 7]
let empty: List<Int> = []
print("x:", [a[i] for i in at..3])
print(at, empty, start, [a[i] for i in from..start], empty, from)
let r = 2..5
print(r.first, r.last)
let sep = ", "
let item = "b"
let first = "a"
print(sep.join([first, item]))
let x = 0
let i = 1
let idx = 0
let seq = [1, 2]
let (value, result, tests, row, scope) = (9, 1, 2, 3, 4)
print(a.get(i) ?? x, seq.get(idx) ?? x, value + result + tests + row + scope)
let (lo, hi) = (7, 8)
let got: ?Int = read(none: x)
let span: Range<Int> = read()
print(got ?? lo, span.first, span.last, hi)
