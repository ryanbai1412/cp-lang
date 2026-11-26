// expect: ok, output "3 12 14 2 2 0"
// rule: R07.2.15
// A range is keyed by its values and has no source, so `r[a..b]` is the sub-range `a..b`, itself a range.
let t = (10..20)[12..=14]
let u: Range<Int> = (10..20)[..12]
print(t.len, t.first!, t.last!, u.len, (10..20)[18..].len, (10..20)[20..].len)
