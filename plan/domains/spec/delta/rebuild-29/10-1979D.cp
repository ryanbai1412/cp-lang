// CF 1979D Fixing a Binary String. After the operation the string is s[p..] then s[..p]
// reversed, and a k-proper string is one of two targets t (blocks of k, starting with 0 or 1).
// So s[p..] must be the head t[..n-p] (z-function of t + s) and s[..p] must be the head of
// t reversed (common prefix of s and t.rev).

fn flip(c: Char) -> Char = if c == '0': '1' else: '0'

@!multitest
let (n, k): (2..=1e5, 1..=n) = read()
let s = read<Str>()
var ans = -1
for c in ['0', '1']:
    let t = Str([if (i / k) % 2 == 0: c else: flip(c) for i in 0..n])
    let tail = s.common_prefix_len(t.rev)
    let z = z_function(t + s)
    if let p = (1..=n).find(p => p <= tail and (p == n or z[n + p] >= n - p)):
        ans = p
        break
print(ans)

@check
fn proper():
    let (n, k): (2..=1e5, 1..=n) = read()
    let s = read<Str>()
    let p: Int = read_output()
    fn good(p: Int) -> Bool:
        let u = List(s[p..]) + List(s[..p].rev)
        (0..k).all(i => u[i] == u[0]) and (0..n - k).all(i => u[i + k] != u[i])
    if p == -1: assert not (1..=n).any(good)
    else: assert p in 1..=n and good(p)
