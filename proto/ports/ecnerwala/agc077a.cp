// AtCoder AGC 077 A: recursively split mismatched AB intervals via binary search
// on balanced B-count prefixes and emit pre-/post-reversal operations.
struct Seg:
    l: Int
    r: Int
    d: Int

fn seg_len(s: Seg) -> Int = (s.r - s.l) * s.d + 1
fn seg_flip(s: Seg) -> Seg = Seg(s.r, s.l, -s.d)
fn seg_with(s: Seg, k: Int) -> Seg = Seg(s.l, s.l + s.d * (k - 1), s.d)
fn seg_without(s: Seg, k: Int) -> Seg = Seg(s.l + s.d * k, s.r, s.d)
fn seg_step(s: Seg) -> Seg = Seg(s.l + s.d, s.r, s.d)

@!multitest
let n: Int = read()
let xs: Str = read()
let ys: Str = read()

let cnt_x = count(xs[i] == 'B' for i in 0..n)
let cnt_y = count(ys[i] == 'B' for i in 0..n)

if cnt_x != cnt_y:
    print("No")
elif xs == ys:
    print("Yes")
    print(0)
else:
    var l = 0
    var r = n - 1
    while xs[l] == ys[l]: l += 1
    while xs[r] == ys[r]: r -= 1
    if xs[l] != 'A' or xs[r] != 'B':
        print("No")
    else:
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
            while seg_len(x) > 0 and xs[x.l] == ys[y.l]:
                x = seg_step(x)
                y = seg_step(y)
                pre = seg_step(pre)
                post = seg_step(post)
            if seg_len(x) == 0: return
            let dir = Int(xs[x.l] == 'B') - Int(ys[y.l] == 'B')
            var mi = 1
            var ma = seg_len(x)
            while ma - mi > 1:
                let md = (mi + ma) / 2
                let vx = get_px(seg_with(x, md))
                let vy = get_py(seg_with(y, md))
                if (vx - vy) * dir > 0: mi = md
                else: ma = md
            rec(seg_without(x, ma), seg_without(pre, ma), seg_without(post, ma), seg_without(y, ma))
            x = seg_with(x, ma)
            pre = seg_with(pre, ma)
            post = seg_with(post, ma)
            y = seg_with(y, ma)
            if xs[x.l] == 'B':
                x = seg_flip(x)
                pre = seg_flip(pre)
                post = seg_flip(post)
                y = seg_flip(y)
            if pre.d == 1:
                pre_ops.push((pre.l, pre.r))
                x = seg_flip(x)
            else:
                post_ops.push((post.l, post.r))
                y = seg_flip(y)
            rec(x, pre, post, y)

        rec(Seg(l + 1, r - 1, 1), Seg(l + 1, r - 1, 1), Seg(r - 1, l + 1, -1), Seg(r - 1, l + 1, -1))
        pre_ops.push((l, r))
        for op in post_ops.rev: pre_ops.push(op)
        print("Yes")
        print(pre_ops.len)
        for (a, b) in pre_ops: print(a + 1, b + 1)
