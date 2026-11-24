// `read<T>(from: s)` runs on the VM through the same read as standard
// input: a word, a tuple, a bounded int, an optional and an array whose
// bounds come from registers, with `None` for a failed or partial read.
let n: 1..=10 = read()
let a = read<Int>(from: "12")!
let p = read<(Int, Str)>(from: " 4 x\n")!
let k = read<1..=n>(from: "3")
let far = read<1..=n>(from: "99")
let left = read<Int>(from: "4 5")
var sum = 0
for i in 0..n:
    let v = read<Int>(from: "{i * i}")!
    sum += v
let after: Int = read()
print(a, p, k!, yes(far == None), yes(left == None), sum, after)
