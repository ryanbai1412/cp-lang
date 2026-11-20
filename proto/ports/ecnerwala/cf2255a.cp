// Codeforces 2255 A: on a circle of 2n cells, a cell counts when it is a 1
// followed by a 1, or a 0 preceded by a 1.  Report the counts on odd and on
// even positions.
@!multitest
let (n, k): (1..=2e5, Int) = read()
let s: Str = read()
let m = 2 * n
var hit = [0, 0]
for i in 0..m:
    let after = s[(i + 1) % m]
    let before = s[(i - 1) % m]
    if (s[i] == '1' and after == '1') or (s[i] == '0' and before == '1'): hit[i % 2] += 1
print(hit[1], hit[0])
