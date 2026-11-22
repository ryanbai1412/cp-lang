// expect: ok, output "0 154"
use std.heap.{MinMaxHeap}
var h: MinMaxHeap<Int> = MinMaxHeap()
var ref: List<Int> = []
var seed = 12345
var bad = 0
for step in 0..600:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let r = seed % 7
    if r < 4 or ref.len == 0:
        let x = seed / 7 % 50
        h.push(x)
        ref.push(x)
        ref = ref.sorted()
    else:
        if r == 4:
            if h.pop_first() != Some(ref[0]): bad += 1
            var nr: List<Int> = []
            for i in 1..ref.len: nr.push(ref[i])
            ref = nr
        else:
            if h.pop_last() != Some(ref[ref.len - 1]): bad += 1
            var nr: List<Int> = []
            for i in 0..ref.len - 1: nr.push(ref[i])
            ref = nr
    if ref.len > 0:
        if h.first != Some(ref[0]) or h.last != Some(ref[ref.len - 1]) or h.len != ref.len: bad += 1
print(bad, ref.len)
