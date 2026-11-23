// Codeforces 1976C Job Interview
// in: t; per case n m, then n+m+1 programming skills a, then n+m+1 testing skills b. out: per case n+m+1 team skills, one per absent candidate
@!multitest
let (n, m) = read<(0..=2e5, 0..=2e5)>()
let k = n + m + 1
let a = read<[k]1..=1e9>()
let b = read<[k]1..=1e9>()

fn hire(programmers: Int, testers: Int) -> (List<Bool>, Int):
    var (ps, ts, total) = (programmers, testers, 0)
    var as_prog: List<Bool> = []
    for i in 0..k:
        let prog = (a[i] > b[i] and ps > 0) or ts == 0
        if prog:
            ps -= 1
            total += a[i]
        else:
            ts -= 1
            total += b[i]
        as_prog.push(prog)
    (as_prog, total)

let (prog1, total1) = hire(n + 1, m)
let (_, total2) = hire(n, m + 1)
print([if prog1[i]: total1 - a[i] else: total2 - b[i] for i in 0..k])
