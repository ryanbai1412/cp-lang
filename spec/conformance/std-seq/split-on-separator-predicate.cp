// expect: ok, output "1 2 0 1\n2"
// rule: 07.3.1
// `a.split(sep: p)` from the sequence-patterns table needs no `use` (decision 150) and walks the pieces, on a list and a `Str` alike.
let a = [1, 0, 2, 3, 0, 0, 4]
print([p.len for p in a.split(sep: x => x == 0)])
print([w.len for w in "ab cd".split(sep: c => c == ' ')].len)
