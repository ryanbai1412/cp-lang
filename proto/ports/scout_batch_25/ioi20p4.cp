// IOI '20 P4 - Packing Biscuits (DMOJ: ioi20p4)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// a[i] biscuits of tastiness 2^i (i < k <= 60). Pack x bags, every bag with
// the same total tastiness y. How many values of y are possible? The total
// tastiness in the pantry is at most 1e18, x <= 1e18, q <= 1000 calls.
//
// Let s[i] = sum of a[j] * 2^j for j <= i. A y whose highest bit is i needs
// y <= s[i] / x, and the rest of y (below bit i) is the same question with a
// smaller cap. With f(c) = number of valid y < c:
//     f(c) = f(2^i) + f(min(c, s[i] / x + 1) - 2^i),   i = top bit of c - 1
// The first term is always a power of two, so tabulate pow[i] = f(2^i) from
// the bottom; the second term is then a single chain, no memo needed.
@grader
fn count_tastiness(x: 1..=1e18, a: List<0..=1e18>) -> Int:
    const K = 60
    let s = scan([if i < a.len: a[i] << i else: 0 for i in 0..K], +)

    var pow = [1]                                  // pow[i] = f(2^i)
    fn below(cap: Int) -> Int:
        if cap <= 1: return max(cap, 0)
        let i = bit_len(cap - 1) - 1
        pow[i] + below(min(cap, s[i] / x + 1) - (1 << i))
    for i in 0..K: pow.push(below(1 << (i + 1)))
    pow[K]

@brute(count_tastiness)
fn slow(x: Int, a: List<Int>) -> Int
    where a.len <= 6 and a.sum() <= 40:
    // y works iff, taking bits of y from the top, the biscuits of type <= i
    // not yet promised can cover x bags of 2^i.
    let total = sum(a[i] << i for i in 0..a.len)
    count(y for y in 0..=total / x if feasible(x, a, y))

fn feasible(x: Int, a: List<Int>, y: Int) -> Bool:
    var carry = 0
    for i in 0..a.len.max(bit_len(y)):
        let have = carry + (if i < a.len: a[i] else: 0)
        let need = if y >> i & 1 == 1: x else: 0
        if have < need: return false
        carry = (have - need) / 2
    true
