// Sliding window maximum: n integers and a window length k; print the maximum
// of each of the n - k + 1 windows. `max` is Selective, so Window keeps a monotone deque.
let n = read<1..=1e6>()
let k = read<1..=n>()
domain I = 0..n
let a = read<[I]-1e9..=1e9>()
var w = Window(max)
var out = List<Int>()
for i in I:
    w.push(a[i])
    if w.len > k: w.pop()
    if w.len == k: out.push(w.all!)
print(out)
