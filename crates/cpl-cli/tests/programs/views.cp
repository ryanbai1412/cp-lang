use std.strings.{prefix_function, z_function, is_palindrome}

// O(1) slice and reverse views over Str and List
let s = "abracadabra"
let sv = s[1..6]
let sr = s.rev
let srv = s.rev[1..6]
let svr = s[1..6].rev
let srr = s.rev.rev

print(sv, sv.len, sv.is_empty, sv[0], sv[4], sv.first, sv.last, sv.get(2), sv.get(99))
print(sr, sr[0], sr[sr.len - 1], sr.first, sr.last)
print(srv, svr, srr == s, sv == "braca", sr == "arbadacarba", sv < "bracb", "braca" == sv)
print(s[0..0].is_empty, s[0..0].first, s[0..0].last, s[1..4] == s[8..11])
print(s[1..6].count('a'), s[1..6].contains('c'), "rac" in s[1..6], "xyz" in s[1..6])

var acc = ""
for c in s[1..5].rev:
    acc.push(c)
print(acc)

// Hashing: HashMap / HashSet lookup with views
var seen_s = HashMap<Str, Int>()
seen_s["braca"] = 42
seen_s["arbadacarba"] = 99
print(seen_s.get(s[1..6]), seen_s.get(s.rev), s[1..6] in seen_s)

var seen_l = HashMap<List<Int>, Int>()
seen_l[[20, 30, 40, 50]] = 7
print(seen_l.get([10, 20, 30, 40, 50, 60][1..5]))

let owned_s: Str = s[1..6].to_str()
let owned_sr: Str = s.rev.to_str()
print(owned_s, owned_sr)

// List views: slice, reverse, nested, indexing, iteration, comparison, explicit copy
let xs = [10, 20, 30, 40, 50, 60]
let lv = xs[1..5]
let lr = xs.rev
let lrv = xs.rev[1..4]
let lvr = xs[1..5].rev
let lrr = xs.rev.rev

print(lv, lv.len, lv.is_empty, lv[0], lv[3], lv.first, lv.last, lv.get(1), lv.get(10))
print(lr, lr[0], lr[5], lrv, lvr)
print(lrr == xs, lv == [20, 30, 40, 50], lvr == [50, 40, 30, 20], lv < [20, 30, 40, 51])
print(xs[2..2].is_empty, xs[2..2].first, xs[2..2].last)
print(xs[1..5].contains(30), xs[1..5].contains(99))

var tot = 0
for (i, x) in xs[1..4].rev.items:
    tot += (i + 1) * x
print(tot)

let owned_l: List<Int> = xs[1..4].to_list()
let owned_lr: List<Int> = xs.rev.to_list()
print(owned_l, owned_lr)

// Seq / Indexed stdlib functions directly on views
print(sum(xs[1..5]), min(xs.rev), max(xs[0..4]))
print(runs("aabbbcc"[1..6]))
print(prefix_function("ababa"[0..5]))
print(z_function("ababa".rev))
print(is_palindrome("xracecary"[1..8]), is_palindrome("abcdef"[1..5]))

// Fused concat indexing and view-concat printing, plus clear-reuse
var left: List<Int> = [1, 2, 3]
var right: List<Int> = [4, 5]
print((left.rev + right)[0], (left.rev + right)[2], (left.rev + right)[4])
print(s[0..4] + s[5..11])
left = []
right = []
left.push(7)
print(left, right.len)

// First-class .rev view over a 1D domain array (and struct-held domain array)
domain Node = 1..=4
struct Holder(at: [Node]Int)
let arr: [Node]Int = [Node] i => (i.ord + 1) * 10
let arr_rev = arr.rev
print(arr_rev.len, arr_rev[0], arr_rev[3], arr_rev.first!, arr_rev.last!)
var arr_acc = 0
for x in arr_rev:
    arr_acc = arr_acc * 100 + x
print(arr_acc)
print(arr_rev.rev.to_list())
print(arr.rev[1..3].to_list())
let h = Holder(arr)
let post = h.at.rev
var post_sum = 0
for x in post:
    post_sum = post_sum * 100 + x
print(post_sum, post.len)
let node_rev = Node.rev
var node_sum = 0
for u in node_rev:
    node_sum = node_sum * 10 + u.ord + 1
print(node_sum)
