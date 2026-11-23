// cf1523d Love-Hate, revised: among the largest sets, prefer the one liked by the most friends,
// and print that count on a second line.
let n: 1..=2e5 = read()
let m: 1..=60 = read()
let p: 1..=15 = read()
domain F = 1..=n
domain Cur = 0..m
let likes = read<[F, Cur]Char>()
let need = (n + 1) / 2

var rng = Rng(1523)
var best: [Cur]Bool = false
var best_key = (0, 0)
for _ in 0..30:
    let f = F(rng.int(1, n))
    let mine = [c for c in Cur if likes[f, c] == '1']
    domain B = 0..mine.len
    var cnt: [B.subsets]Int = 0
    for g in F:
        var s = Subset<B>.empty
        for b in B if likes[g, mine[b.ord]] == '1': s = s.with(b)
        cnt[s] += 1
    for b in B:
        for s in B.subsets if b not in s: cnt[s] += cnt[s.with(b)]
    for s in B.subsets if cnt[s] >= need and (s.len, cnt[s]) > best_key:
        best_key = (s.len, cnt[s])
        best = [Cur] c => false
        for b in s.items: best[mine[b.ord]] = true
print(Str(if best[c]: '1' else: '0' for c in Cur))
print(best_key.1)
