// fixed: i
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let even = (x: Int) => x % 2 == 0 // within: seq.partition_in_place
let even2 = (x: Int) => x % 2 == 0 // within: seq.stable_partition_in_place
let digits = (x: Int) => x // within: seq.radix_sort
var s = 0
for r in 0..16:
    var a = xs.clone()
    s += a.partition_in_place(even) // timed: seq.partition_in_place
    s += a.stable_partition_in_place(even2) // timed: seq.stable_partition_in_place
    a.radix_sort(digits) // timed: seq.radix_sort
print(s)
