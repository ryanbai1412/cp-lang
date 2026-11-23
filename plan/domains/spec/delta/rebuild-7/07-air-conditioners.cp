// Codeforces 1547E Air Conditioners
// in: t (blank line before each test); per test n k, positions a_1..a_k, temperatures t_1..t_k. out: per test n temperatures on one line.
@!multitest
let n: 1..=3e5 = read()
domain C = 1..=n
let k: 1..=n = read()
let at = read<[k]C>()
let temp = read<[k]1..=1e9>()
var best: [C]?Int = [C] _ => None
for (c, t) in zip(at, temp): best[c] min= t
var run: ?Int = None
for c in C:
    best[c] min= run + 1
    run = best[c]
run = None
for c in C.rev:
    best[c] min= run + 1
    run = best[c]
print(best!)
