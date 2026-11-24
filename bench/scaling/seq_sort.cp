// fixed: i
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let ys = [i for i in 0..n]
let even = (x: Int) => x % 2 == 0 // within: seq.partition_in_place
let even2 = (x: Int) => x % 2 == 0 // within: seq.stable_partition_in_place
let digits = (x: Int) => x // within: seq.radix_sort
let low = (x: Int) => x % 64 // within: seq.group
var s = 0
for r in 0..1024:
    var a = xs.clone() // timed: seq.clone
    a.dedup() // timed: seq.dedup
    a.reverse() // timed: seq.reverse
    a.rotate_left(n / 3) // timed: seq.rotate_left
    a.rotate_right(n / 3) // timed: seq.rotate_right
    s += a.resized(n + 1, 0).len // timed: seq.resized
    a.insert(0, 5) // timed: seq.insert
    s += a.remove(0) // timed: seq.remove
for r in 0..256:
    var a = xs.clone()
    s += a.rotated_left(n / 3).len // timed: seq.rotated_left
    s += a.rotated_right(n / 3).len // timed: seq.rotated_right
    s += a.merge_sorted(ys).count() // timed: seq.merge_sorted
    s += a.sorted_union(ys).len // timed: seq.sorted_union
    s += a.sorted_intersect(ys).len // timed: seq.sorted_intersect
    s += a.sorted_diff(ys).len // timed: seq.sorted_diff
for r in 0..64:
    s += xs.lis().len // timed: seq.lis
    s += xs.spans().len // timed: seq.spans
    s += xs.prev_smaller().len // timed: seq.prev_smaller
    s += xs.next_smaller().len // timed: seq.next_smaller
    s += xs.perm_order() // timed: seq.perm_order
    s += xs.perm_sign() // timed: seq.perm_sign
for r in 0..16:
    var a = xs.clone()
    s += a.partition_in_place(even) // timed: seq.partition_in_place
    s += a.stable_partition_in_place(even2) // timed: seq.stable_partition_in_place
    a.radix_sort(digits) // timed: seq.radix_sort
    var b = xs.clone()
    b.sort() // timed: seq.sort
    s += xs.group(low).len // timed: seq.group
    s += xs.prev_same().len // timed: seq.prev_same
    s += xs.next_same().len // timed: seq.next_same
print(s)
