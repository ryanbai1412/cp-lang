// Every in-place sequence mutator works uniformly over `IndexedMut` (`List<T>`, `[D]T`, `[1..=n]T`, `[V.cuts]T`, `Str`).
use std.rand.Rng
use std.seq.{
    next_permutation, nth_element, prev_permutation, reverse, rotate_left, rotate_right,
    scan_in_place, suffix_scan_in_place,
}

domain V = 0..5

// 1. Nominal domain array `[V]Int`
var a: [V]Int = [V] i => [30, 10, 50, 20, 40][i.ord]
nth_element(mut a, 2)
print(a[V(2)])
a.sort()
print(a)
reverse(mut a, range: 1..4)
print(a)
a.rotate_left(2)
print(a)
a.rotate_right(1, range: 0..3)
print(a)

var p: [V]Int = [V] i => i.ord + 1
let has_next = next_permutation(mut p)
print(has_next, p)
let has_prev = prev_permutation(mut p)
print(has_prev, p)

var rng1 = Rng.new(42)
var rng2 = Rng.new(42)
var p_arr: [V]Int = [V] i => i.ord
var p_list: List<Int> = [0, 1, 2, 3, 4]
rng1.shuffle(mut p_arr)
rng2.shuffle(mut p_list)
print([p_arr[i] for i in V] == p_list, p_arr)

var s_arr: [V]Int = [V] i => [5, 2, 8, 1, 6][i.ord]
s_arr.scan_in_place(min)
print(s_arr)
s_arr.suffix_scan_in_place(max)
print(s_arr)

// 2. 1-based range array `[1..=5]Int`
var one: [1..=5]Int = 0
for i in 1..=5: one[i] = i * 10
one.rotate_left(2)
print(one)
one.scan_in_place(+)
print(one)

// 3. Cut-indexed array `[V.cuts]Int`
var cut: [V.cuts]Int = [V.cuts] c => 10 - c.ord
cut.swap(V(0).l, V(4).r)
cut.scan_in_place(min)
print(cut)

// 4. Mutable `Str`
var s = "abcde"
s.rotate_left(2)
print(s)
reverse(mut s, range: 1..4)
print(s)
let sn = next_permutation(mut s)
print(sn, s)
var rng3 = Rng.new(7)
rng3.shuffle(mut s)
print(s)
