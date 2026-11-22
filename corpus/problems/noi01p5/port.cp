// cp: 0.1 1f1a960+dirty
// NOI 2001 P5: count x in [1..=m]^n with sum of k_i * x_i^p_i == 0.
// Meet in the middle: all sums of the first half, then look up each
// negated sum of the second half.
let n: 1..=6 = read()
let m: 1..=150 = read()
domain T = 0..n
let terms: [T](k: -20..=20, p: 1..=3) = read()

fn pow(x: 1..=150, p: 1..=3) -> 1..=3375000:
    product(x for _ in 0..p)

// Every value of sum k_t * x_t^p_t over the given terms.
fn sums(ts: Range<T>) -> List<Int>:
    var cur: List<Int> = [0]
    for t in ts:
        cur = [s + terms[t].k * pow(x, terms[t].p) for s in cur for x in 1..=m]
    cur

type Cut = T.cuts
let mid = Cut(n / 2)
let seen = counts(sums(..mid))
print(sum(seen[-s] for s in sums(mid..)))
