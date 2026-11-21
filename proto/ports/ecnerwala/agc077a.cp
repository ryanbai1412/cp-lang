// AtCoder AGC 077 A.
// Reversing a substring starting with 'A' and ending with 'B' preserves B-counts.
// Whenever two equal-B-count segments differ at their first character, their
// running B-count difference starts at ±1 and returns to 0 at the end; binary
// search finds the first balanced prefix `ma`, splitting into an independent
// suffix (`drop(ma)`) and a balanced prefix (`take(ma)`) that starts with 'A'
// and ends with 'B' (after orienting) so one pre- or post-reversal fixes its ends.
use std.prefix.{Prefix1, prefix1}

struct Seg(l: Int, r: Int, d: Int):
    @get fn len(self) -> Int = (r - l) * d + 1
    fn flip(self) -> Seg = Seg(r, l, -d)
    fn take(self, k: Int) -> Seg = Seg(l, l + d * (k - 1), d)
    fn drop(self, k: Int) -> Seg = Seg(l + d * k, r, d)
    fn step(self) -> Seg = Seg(l + d, r, d)

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
let xs: Str = read()
let ys: Str = read()

if xs.count('B') != ys.count('B'):
    print("No")
    return
if xs == ys:
    print("Yes")
    print(0)
    return

var (l, r) = (0, n - 1)
while xs[l] == ys[l]: l += 1
while xs[r] == ys[r]: r -= 1
if xs[l] != 'A' or xs[r] != 'B':
    print("No")
    return

let pref_x = prefix1([Pos] p => Int(xs[p.ord] == 'B'))
let pref_y = prefix1([Pos] p => Int(ys[p.ord] == 'B'))

fn count_b(pref: Prefix1<Pos, Int>, s: Seg) -> Int =
    if s.d == 1: pref.sum(Pos(s.l + 1)..=Pos(s.r + 1)) else: pref.sum(Pos(s.r + 1)..=Pos(s.l + 1))

var pre_ops: List<(Pos, Pos)> = []
var post_ops: List<(Pos, Pos)> = []

fn solve_seg(var x: Seg, var pre: Seg, var post: Seg, var y: Seg):
    while x.len > 0 and xs[x.l] == ys[y.l]:
        (x, y, pre, post) = (x.step(), y.step(), pre.step(), post.step())
    if x.len == 0: return
    let dir = Int(xs[x.l] == 'B') - Int(ys[y.l] == 'B')
    let ma = bsearch_last(
        1..x.len,
        md => (count_b(pref_x, x.take(md)) - count_b(pref_y, y.take(md))) * dir > 0,
    )! + 1
    solve_seg(x.drop(ma), pre.drop(ma), post.drop(ma), y.drop(ma))
    (x, pre, post, y) = (x.take(ma), pre.take(ma), post.take(ma), y.take(ma))
    if xs[x.l] == 'B':
        (x, pre, post, y) = (x.flip(), pre.flip(), post.flip(), y.flip())
    if pre.d == 1:
        pre_ops.push((Pos(pre.l + 1), Pos(pre.r + 1)))
        x = x.flip()
    else:
        post_ops.push((Pos(post.l + 1), Pos(post.r + 1)))
        y = y.flip()
    solve_seg(x, pre, post, y)

solve_seg(Seg(l + 1, r - 1, 1), Seg(l + 1, r - 1, 1), Seg(r - 1, l + 1, -1), Seg(r - 1, l + 1, -1))
pre_ops.push((Pos(l + 1), Pos(r + 1)))
for op in post_ops.rev: pre_ops.push(op)
print("Yes")
print(pre_ops.len)
for (a, b) in pre_ops: print(a, b)
