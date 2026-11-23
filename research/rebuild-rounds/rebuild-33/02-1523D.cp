// cf1523d Love-Hate: the answer is liked by some friend among the >= ceil(n/2) who like it,
// so a random friend is in that group with probability >= 1/2. Restrict every friend to that
// friend's <= 15 currencies, count masks, and take superset sums over `B.subsets`.
let n: 1..=2e5 = read()
let m: 1..=60 = read()
let p: 1..=15 = read()
domain F = 1..=n
domain Cur = 0..m
let likes = read<[F, Cur]Char>()
let need = (n + 1) / 2

var rng = Rng(1523)
var best: [Cur]Bool = false
var best_len = 0
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
    for s in B.subsets if cnt[s] >= need and s.len > best_len:
        best_len = s.len
        best = [Cur] c => false
        for b in s.items: best[mine[b.ord]] = true
print(Str(if best[c]: '1' else: '0' for c in Cur))
