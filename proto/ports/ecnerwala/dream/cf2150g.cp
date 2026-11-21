// Codeforces 2150 G: Counting Is Fun: The Finale.
//
// b > a means b = a[..i] + '1' + r for some i with a[i] = '0'. b is good iff
// its shortest prefix with LNDS >= k leaves a suffix with LNDS >= k. Two
// facts carry the count:
// - z zeros and o ones make a string with LNDS >= m in exactly
//       long_lnds(z, o, m) = C(z + o, max(z, o, m))
//   ways;
// - LNDS(s + r) = max(s.lnds + ones(r), s.zeros + LNDS(r)), so r completes s
//   to LNDS >= k iff ones(r) >= k - s.lnds or LNDS(r) >= k - s.zeros.
//
// Walk a in two phases. While a's prefix has LNDS < k (Seeking), each flip
// gives s, and the good b are summed over the state h where s + r first
// reaches LNDS k, weighted long_lnds of what remains after h. With e0 = k -
// s.zeros and e1 = k - s.lnds, the first hits d letters after s lie on one
// anti-diagonal with two ballot counts:
//     top(d)  = C(d - 1, e1 - 1) - C(d - 1, e0)  at the state with e1 more ones,
//     main(d) = C(d - 1, e0 - 1) - C(d - 1, e0)  at every later
// state on that diagonal, up to k zeros. Each is the walks whose d-th letter
// completes the threshold (e1 ones, resp. e0 zeros), less by reflection the
// walks that reached the zero threshold earlier (brute force agrees for
// x, y <= 5).
// Record each as a ray and sweep once, O(k^2).
// Once a's prefix reaches k (Split), restart the walk there; each flip then
// needs only a long suffix, which is the completion count.
@!multitest
type Z = Mod<998_244_353>

let (x, y, k): (1..=5000, 1..=5000, 1..=9999) = read()
let a: Str = read()
let comb = Z.comb(x + y)

struct Walk(zeros: Int, ones: Int, lnds: Int):
    fn push(self, c: Char) -> Walk =
        if c == '0': Walk(zeros + 1, ones, max(lnds, zeros + 1))
        else: Walk(zeros, ones + 1, lnds + 1)

enum Phase:
    Seeking
    Split(base: Walk)

fn long_lnds(z: Int, o: Int, m: Int) -> Z = comb.choose(z + o, max(z, o, m))
fn completions(s: Walk, z: Int, o: Int) -> Z =
    if o >= k - s.lnds: comb.choose(z + o, z) else: long_lnds(z, o, k - s.zeros)

domain Cnt = 0..=k
var rays: [Cnt, Cnt]Z = 0
// w first hits at (z, o) and at every (z + t, o - t) after it.
fn ray(z: Int, o: Int, w: Z): rays[z, o] += w
fn first_hits_from(s: Walk):
    if s.lnds == k:
        ray(s.zeros, s.ones, 1)
        ray(s.zeros + 1, s.ones - 1, -1)
        return
    let (e0, e1) = (k - s.zeros, k - s.lnds)
    for d in e1..e0 + e1:
        let top = comb.choose(d - 1, e1 - 1) - comb.choose(d - 1, e0)
        let main = comb.choose(d - 1, e0 - 1) - comb.choose(d - 1, e0)
        let (z, o) = (s.zeros + d - e1, s.ones + e1)
        ray(z, o, top)
        ray(z + 1, o - 1, main - top)

var ans: Z = 0
var walk = Walk(0, 0, 0)
var phase = Seeking
for c in a:
    if c == '0':
        let s = walk.push('1')
        match phase:
            Seeking => first_hits_from(s)
            Split(base) => ans += completions(s, x - base.zeros - s.zeros, y - base.ones - s.ones)
    walk = walk.push(c)
    if let Seeking = phase, walk.lnds >= k:
        phase = Split(base: walk)
        walk = Walk(0, 0, 0)

for z in 0..k, o in 1..=k:
    rays[z + 1, o - 1] += rays[z, o]
// rays[z, o] is now the number of first hits at (z, o).
print(ans + sum(rays[z, o] * long_lnds(x - z, y - o, k) for z in Cnt, o in Cnt))
