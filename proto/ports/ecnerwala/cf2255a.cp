// Codeforces 2255 A: on a circle of 2n cells, a cell counts when it is a 1
// followed by a 1, or a 0 preceded by a 1. Report the counts on odd and on
// even positions.
@!multitest
let (n, _k): (1..=2e5, Int) = read()
let s: Str = read()
let m = 2 * n

fn hit(i: Int) -> Bool:
    (s[i] == '1' and s[(i + 1) % m] == '1') or (s[i] == '0' and s[(i - 1) % m] == '1')

print(count(hit(i) for i in 1..m by 2), count(hit(i) for i in 0..m by 2))
