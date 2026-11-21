// AtCoder AGC 077 A: recursively split mismatched AB intervals via binary search
// on balanced B-count prefixes and emit pre-/post-reversal operations.
struct Seg(l: Int, r: Int, d: Int):
    @get fn len(self) -> Int = (r - l) * d + 1
    fn flip(self) -> Seg = Seg(r, l, -d)
    fn take(self, k: Int) -> Seg = Seg(l, l + d * (k - 1), d)
    fn drop(self, k: Int) -> Seg = Seg(l + d * k, r, d)
    fn step(self) -> Seg = Seg(l + d, r, d)

@!multitest
let n: Int = read()
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

var pref_x: [0..=n]Int = 0
var pref_y: [0..=n]Int = 0
for i in 0..n:
    pref_x[i + 1] = pref_x[i] + Int(xs[i] == 'B')
    pref_y[i + 1] = pref_y[i] + Int(ys[i] == 'B')

fn get_px(s: Seg) -> Int = if s.d == 1: pref_x[s.r + 1] - pref_x[s.l] else: pref_x[s.l + 1] - pref_x[s.r]
fn get_py(s: Seg) -> Int = if s.d == 1: pref_y[s.r + 1] - pref_y[s.l] else: pref_y[s.l + 1] - pref_y[s.r]

var pre_ops: List<(Int, Int)> = []
var post_ops: List<(Int, Int)> = []

fn rec(var x: Seg, var pre: Seg, var post: Seg, var y: Seg):
    while x.len > 0 and xs[x.l] == ys[y.l]:
        (x, y, pre, post) = (x.step(), y.step(), pre.step(), post.step())
    if x.len == 0: return
    let dir = Int(xs[x.l] == 'B') - Int(ys[y.l] == 'B')
    let ma = bsearch_last(1..x.len, md => (get_px(x.take(md)) - get_py(y.take(md))) * dir > 0)! + 1
    rec(x.drop(ma), pre.drop(ma), post.drop(ma), y.drop(ma))
    (x, pre, post, y) = (x.take(ma), pre.take(ma), post.take(ma), y.take(ma))
    if xs[x.l] == 'B':
        (x, pre, post, y) = (x.flip(), pre.flip(), post.flip(), y.flip())
    if pre.d == 1:
        pre_ops.push((pre.l, pre.r))
        x = x.flip()
    else:
        post_ops.push((post.l, post.r))
        y = y.flip()
    rec(x, pre, post, y)

rec(Seg(l + 1, r - 1, 1), Seg(l + 1, r - 1, 1), Seg(r - 1, l + 1, -1), Seg(r - 1, l + 1, -1))
pre_ops.push((l, r))
for op in post_ops.rev: pre_ops.push(op)
print("Yes")
print(pre_ops.len)
for (a, b) in pre_ops: print(a + 1, b + 1)
