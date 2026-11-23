// cf1948d Tandem Repeats?: a repeat of half-length d is a run of at least d
// positions i where s[i] and s[i + d] can match. Try d from the largest down.
@!multitest
let s = read<Str>()
let n = s.len
fn fits(x: Char, y: Char) -> Bool = x == y or x == '?' or y == '?'
let d = (1..=n / 2).rev.find(d =>
    (0..n - d).map(i => fits(s[i], s[i + d])).runs().any(r => r.first! and r.len >= d))
print(2 * (d ?? 0))
