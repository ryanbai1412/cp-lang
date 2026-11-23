// Runnable port (proto/cp.py lib-run) of etsr23r1p3.cp
// original: corpus/submissions/5494303.cpp

let tcs: Int = read()
for _ in 0..tcs:
    let m: Int = read()
    let s: Str = read()
    if m >= 22:
        print("NO")
        continue
    let n = s.len
    domain Alpha = 0..m
    domain Pos = 0..=n
    var prv: [Pos][Alpha]?Int = [Pos] _ => [Alpha] _ => None
    for i in 0..n:
        for c in Alpha:
            prv[Pos(i + 1)][c] = prv[Pos(i)][c]
        prv[Pos(i + 1)][Alpha(s[i] - 'a')] = i

    let total = 1 << m
    var dp: List<?Int> = List.filled(total, None)
    dp[0] = n
    for mask in 1..total:
        var best: ?Int = n
        for j in 0..m:
            if ((mask >> j) & 1) == 0: continue
            let prev_opt = dp[mask ^ (1 << j)]
            if prev_opt == None:
                best = None
                break
            let hit = prv[Pos(prev_opt!)][Alpha(j)]
            if hit == None:
                best = None
                break
            if best == None or hit! < best!:
                best = hit
        dp[mask] = best

    if dp[total - 1] != None:
        print("YES")
    else:
        print("NO")
