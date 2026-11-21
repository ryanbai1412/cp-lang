// Codeforces 2211 H: Median Deletion.
// Normalize pos[1] < pos[n] by reversing; 1 and n survive in length 2, and every
// other element x survives in length 3, 4, or 5. Outside (pos[1], pos[n]), the
// side holding 1 and n reduces internally to [1, n], while the outer prefix reduces
// to empty (3) iff max_L < x or some value > max_L before 1 can delete max_L
// (breaking any min_L < x between max_L and x first), else 5 if min_L lies left of
// max_L and 4 otherwise. Between pos[1] and pos[n], trapped outer tails add +1 each,
// and an unbroken internal (max_L > x, min_R < x) barrier gives 5.

@!multitest
let n: 1..=2e5 = read()
domain Pos = 0..n
var p: [Pos]1..=n = read()

if n <= 2:
    print([Pos] _ => n)
    return

let rev = p.first_index(x => x == 1)! > p.first_index(x => x == n)!
if rev: p.reverse()

let p1 = p.first_index(x => x == 1)!
let pn = p.first_index(x => x == n)!

fn solve_outer(a: List<Int>) -> List<Int>:
    let m = a.len
    var res = List.filled(m, 3)
    if m == 0: return res
    domain S = 0..m
    let arr: [S]Int = [S] i => a[i.ord]
    let suf_max = arr.suffix_fold(max)
    let seg_min = SegTree<S, Int>(min, arr)
    var (max_val, max_pos) = (arr[S.first], S.first)
    var (min_val, min_pos) = (arr[S.first], S.first)
    for i in S.first + 1..=S.last:
        let x = arr[i]
        if max_val > x:
            let need_break = min_val < x and min_pos > max_pos
            let can_del = if not need_break:
                i < S.last and suf_max[i + 1] > max_val
            else:
                let cut = seg_min.max_right(i + 1, v => v >= min_val)
                cut < S.bounds.last - 1 and suf_max[cut + 1] > max_val
            if not can_del:
                res[i.ord] = if min_val < x and min_pos < max_pos: 5 else: 4
        if max_val max= x: max_pos = i
        if min_val min= x: min_pos = i
    res

var ans: [Pos]Int = 0
ans[p1] = 2
ans[pn] = 2

let left_ans = solve_outer((Pos.first..p1).map(i => p[i]))
for i in Pos.first..p1:
    ans[i] = left_ans[i.ord]

let right_ans = solve_outer((pn + 1..=Pos.last).rev.map(i => n + 1 - p[i]))
for (k, i) in (pn + 1..=Pos.last).rev.items:
    ans[i] = right_ans[k]

let rmq_min = SparseTable<Pos, Int>(min, p)
let rmq_max = SparseTable<Pos, Int>(max, p)
let pref_max = ([Pos] i => (p[i], i)).prefix_fold(max)
let suf_min = ([Pos] i => (p[i], i)).suffix_fold(min)

for i in p1 + 1..pn:
    let x = p[i]
    let (max_l, pos_l) = pref_max[i]
    let (min_r, pos_r) = suf_min[i + 1]
    let left_tail = max_l > x and pos_l < p1
    let right_tail = min_r < x and pos_r > pn
    ans[i] = if left_tail or right_tail:
        3 + Int(left_tail) + Int(right_tail)
    elif max_l > x and min_r < x:
        let break_r = pos_l + 1 < i and rmq_min.fold(pos_l + 1..i) < min_r
        let break_l = i + 1 < pos_r and rmq_max.fold(i + 1..pos_r) > max_l
        if break_r or break_l: 3 else: 5
    else:
        3

if rev: ans.reverse()
print(ans)
