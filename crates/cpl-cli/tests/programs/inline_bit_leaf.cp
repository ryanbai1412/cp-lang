// A leaf fn of popcount and a shift by `i % 64` is inlined at its call, and
// answers as the call does.
struct Bits:
    words: List<U64>
    before: List<Int>
    fn ones(self, i: Int) -> Int:
        let at = i / 64
        before[at] + popcount(words[at] & ((U64(1) << (i % 64)) - U64(1)))
    fn low(self, i: Int) -> Int = ctz(words[i / 64] >> (i % 64))
fn build(n: Int) -> Bits:
    var b = Bits(List.filled(n / 64 + 1, U64(0)), List.filled(n / 64 + 1, 0))
    for i in 0..n:
        if i % 3 == 0: b.words[i / 64] |= U64(1) << (i % 64)
    for k in 1..n / 64 + 1:
        b.before[k] = b.before[k - 1] + popcount(b.words[k - 1])
    b
let b = build(300)
print(b.ones(0), b.ones(1), b.ones(64), b.ones(200), b.ones(299))
print(b.low(0), b.low(1), b.low(130))
