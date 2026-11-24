// expect: ok, output "1 2 2 NO\n0 YES\n0 YES\nYES YES\n3 YES"
// rule: 08.1.2
// A `Slice` keeps its source and the positions `lo..hi`: `len` is `hi - lo`, `is_empty` is `hi <= lo`, and `keys` is the source's keys at those positions, also over a domain.
domain D = 0..5
let a = [5, 6, 7]
let s = a[1..]
print(s.keys.first!, s.keys.last!, s.len, yes(s.is_empty))
let e = a[3..]
print(e.len, yes(e.is_empty))
let none: List<Int> = []
print(none[..].len, yes(none[..].is_empty))
let t = [D] d => d.idx * 10
let u = t.drop(2)
print(yes(u.keys.first! == D(2)), yes(u.keys.last! == D(4)))
print(u.len, yes(u.keys.len == u.len))
