// CF 1979D, revised: print the largest valid p (or -1), not any valid p.
@!multitest
let (n, k): (2..=1e5, 1..=n) = read()
let s: Str = read()

fn find() -> ?Int:
    var best: ?Int = None
    for first in ['0', '1']:
        let target = [if (i / k) % 2 == 0: first else: (if first == '0': '1' else: '0') for i in 0..n]
        let joined = [if i < n: target[i] elif i == n: '#' else: s[i - n - 1] for i in 0..2 * n + 1]
        let z = z_function(joined)
        let rev_target = [target[n - 1 - i] for i in 0..n]
        let agree = s.common_prefix_len(rev_target)
        for p in (1..=min(agree, n)).rev:
            if p == n or z[n + 1 + p] >= n - p:
                best max= p
                break
    best

print(find() ?? -1)
