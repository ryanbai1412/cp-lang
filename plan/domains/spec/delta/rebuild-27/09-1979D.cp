// CF 1979D: after the operation the string is s[p..] + reverse(s[..p]).
// It equals the k-proper target T when s[p..] is a prefix of T (Z function of T#s)
// and s[..p] is a prefix of reverse(T).
// Any valid p is accepted; -1 only when no p works (brute force, for samples).
@check fn valid() -> Bool:
    let (n, k): (2..=1e5, 1..=n) = read()
    let s: Str = read()
    let p: Int = read_output()
    fn proper(q: Int) -> Bool:
        let t = [if i < n - q: s[q + i] else: s[q - 1 - (i - (n - q))] for i in 0..n]
        for i in 0..n:
            if i < k and t[i] != t[0]: return false
            if i + k < n and t[i] == t[i + k]: return false
        true
    if p == -1:
        for q in 1..=n:
            if proper(q): return false
        return true
    1 <= p and p <= n and proper(p)

@!multitest
let (n, k): (2..=1e5, 1..=n) = read()
let s: Str = read()

fn find() -> ?Int:
    for first in ['0', '1']:
        let target = [if (i / k) % 2 == 0: first else: (if first == '0': '1' else: '0') for i in 0..n]
        let joined = [if i < n: target[i] elif i == n: '#' else: s[i - n - 1] for i in 0..2 * n + 1]
        let z = z_function(joined)
        let rev_target = [target[n - 1 - i] for i in 0..n]
        let agree = s.common_prefix_len(rev_target)
        for p in 1..=n:
            if p > agree: break
            if p == n or z[n + 1 + p] >= n - p: return Some(p)
    None

print(find() ?? -1)
