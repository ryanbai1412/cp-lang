// CF 1950G: fewest removals so the rest can be ordered with each neighbour pair
// sharing genre or writer. Mask DP over (set used, last song).
@!multitest
let n: 1..=16 = read()
domain S = 0..n
let songs = read<[S](genre: Str, writer: Str)>()
let ok: [S, S]Bool = [S, S] (i, j) => songs[i].genre == songs[j].genre or songs[i].writer == songs[j].writer

var reach: [S.subsets, S]Bool = false
for i in S: reach[Subset<S>.single(i), i] = true
var most = 0
for s in S.subsets, i in S if i in s and reach[s, i]:
    most max= s.len
    for j in S if j not in s and ok[i, j]: reach[s.with(j), j] = true
print(n - most)
