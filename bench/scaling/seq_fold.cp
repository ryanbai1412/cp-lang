let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let ys = [i for i in 0..n]
let ones = [1 for _ in 0..n]
let os = [Some(i) for i in 0..n]
let thirds = (x: Int) => x % 3 == 0 // within: seq.count
let neg = (x: Int) => x < 0 // within: seq.any
let nonneg = (x: Int) => x >= 0 // within: seq.all
let neg2 = (x: Int) => x < 0 // within: seq.rfind
let last = (x: Int) => x == n - 1 // within: seq.find
let some_neg = (x: Int) => if x < 0: Some(x) else: None // within: seq.first_some
let xor = (a: Int, b: Int) => a ^ b // within: seq.reduce
let xor2 = (a: Int, b: Int) => a ^ b // within: seq.fold_from
let even = (x: Int) => x % 2 == 0 // within: seq.partition
var s = 0
var t = 0.0
for r in 0..1024:
    s += xs.sum() // timed: seq.sum
    if ys.is_sorted(): s += 1 // timed: seq.is_sorted
    s += xs.argmax() ?? 0 // timed: seq.argmax
    s += xs.argmin() ?? 0 // timed: seq.argmin
    s += xs.min() ?? 0 // timed: seq.min
    s += xs.max() ?? 0 // timed: seq.max
    t += xs.mean() ?? 0.0 // timed: seq.mean
    let mm = xs.minmax() // timed: seq.minmax
    s += mm!.0
    if xs.is_subsequence(xs): s += 1 // timed: seq.is_subsequence
    let hi = xs.max_count() // timed: seq.max_count
    let lo = xs.min_count() // timed: seq.min_count
    s += hi!.count + lo!.count
for r in 0..256:
    s += xs.min_k(16).len // timed: seq.min_k
    s += (os.all_some() ?? []).len // timed: seq.all_some
    s += ones.prod() // timed: seq.prod
    s += xs.collect().len // timed: seq.collect
    s += xs.adjacent_diff().sum() // timed: seq.adjacent_diff
    s += xs.fold(max) // timed: seq.fold
    s += xs.deduped().len // timed: seq.deduped
    s += xs.max_k(16).len // timed: seq.max_k
for r in 0..32:
    s += xs.reduce(xor) ?? 0 // timed: seq.reduce
    s += xs.fold_from(0, xor2) // timed: seq.fold_from
    s += xs.count(thirds) // timed: seq.count
    let some = xs.any(neg) // timed: seq.any
    s += Int(some)
    let every = xs.all(nonneg) // timed: seq.all
    s += Int(every)
    s += xs.rfind(neg2) ?? 0 // timed: seq.rfind
    s += ys.find(last) ?? 0 // timed: seq.find
    s += xs.first_some(some_neg) ?? 0 // timed: seq.first_some
    let halves = xs.partition(even) // timed: seq.partition
    s += halves.0.len
    s += xs.distinct().len // timed: seq.distinct
    s += xs.tally().len // timed: hashmap.tally
    if xs.is_permutation(ys): s += 1 // timed: seq.is_permutation
    s += xs.inversions() // timed: seq.inversions
    s += xs.sorted().len // timed: seq.sorted
print(s, t)
