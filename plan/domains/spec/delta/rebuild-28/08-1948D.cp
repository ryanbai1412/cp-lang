// CF 1948D Tandem Repeats?: for each half-length d, run[i] = run[i-1] + 1 while s[i]
// matches s[i+d] ('?' matches anything); a run of length d is a tandem repeat of length 2d.
@!multitest
let s: Str = read()
let n = s.len

fn same(i: Int, j: Int) -> Bool = s[i] == s[j] or s[i] == '?' or s[j] == '?'

fn has_repeat(d: Int) -> Bool:
    var run = 0
    for i in 0..n - d:
        run = if same(i, i + d): run + 1 else: 0
        if run == d:
            return true
    false

print(2 * ((1..=n / 2).rev.find(has_repeat) ?? 0))
