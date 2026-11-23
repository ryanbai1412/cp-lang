// CF 1948D, revised: print the length of a longest tandem repeat and its leftmost 1-based
// start, or 0 when there is none.
@!multitest
let s: Str = read()
let n = s.len

fn same(i: Int, j: Int) -> Bool = s[i] == s[j] or s[i] == '?' or s[j] == '?'

fn repeat_start(d: Int) -> ?Int:
    var run = 0
    for i in 0..n - d:
        run = if same(i, i + d): run + 1 else: 0
        if run == d:
            return Some(i - d + 1)
    None

if let (d, start) = (1..=n / 2).rev.first_some(d => repeat_start(d).map(st => (d, st))):
    print(2 * d, start + 1)
else:
    print(0)
