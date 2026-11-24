// fixed: i
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let ys = [i for i in 0..n]
var s = 0
for r in 0..256:
    var a = xs.clone()
    s += a.rotated_left(n / 3).len // timed: seq.rotated_left
    s += a.rotated_right(n / 3).len // timed: seq.rotated_right
    s += a.sorted_union(ys).len // timed: seq.sorted_union
    s += a.sorted_intersect(ys).len // timed: seq.sorted_intersect
    s += a.sorted_diff(ys).len // timed: seq.sorted_diff
// merge_sorted costs O(1) a step: take one step a call.
for r in 0..1 << 17:
    for x in xs.merge_sorted(ys): // timed: seq.merge_sorted
        s += x
        break
print(s)
