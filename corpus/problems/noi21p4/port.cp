// Source: NOI 2021 P4 (Quantum Communication), https://dmoj.ca/problem/noi21p4
// limits: 3000 ms, 384 MB
// Problem: a dictionary of n <= 4e5 random 256-bit words comes from the statement's generator,
//   seeded with a1, a2. Then m <= 1.2e5 queries, each 64 hex digits and k <= 15: print 1 if a
//   dictionary word differs from the query in at most k bits, else 0. Online: after an answer
//   of 1, every bit of the next query is flipped.
// Solution: cut words into 16 blocks of 16 bits; at most 15 differing bits leave some block
//   equal (pigeonhole). Per block, a table from block value to the words with it; a query compares
//   only the words sharing one of its blocks, by popcount of the xors of their four 64-bit limbs. Random words put
//   about n / 65536 in a bucket: ~100 candidates of 16 popcounts per query.
// original: corpus/submissions/3861375.cpp
domain Block = 0..16
domain Limb = 0..4
domain Half = 0..1 << 16
type Word = [Limb]U64

let (n, m): (Int, Int) = read()
var (k1, k2): (U64, U64) = read()  // up to 2^64 - 1: too big for an Int

/// The statement's generator: an xorshift on two 64-bit words, of which one bit is used.
fn next_bit() -> U64:
    var (k3, k4) = (k1, k2)
    k1 = k4
    k3 ^= k3 << 23
    k2 = k3 ^ k4 ^ (k3 >> 17) ^ (k4 >> 26)
    (k2 + k4) >> 32 & U64(1)

/// The 16-bit block `b` of a word, limb by limb from the high bits.
fn block(w: Word, b: Block) -> Half = Half(
    Int(w[Limb(b.ord / 4)] >> (16 * (3 - b.ord % 4)) & U64(0xFFFF))
)

domain Entry = 0..n
var dict: [Entry]Word = [Entry] e => [Limb] l => U64(0)
var sharing: [Block, Half]List<Entry> = []  // block value -> words
for e in Entry:
    for l in Limb:
        var x = U64(0)
        for _ in 0..64: x = x << 1 | next_bit()
        dict[e][l] = x
    for b in Block:
        let key = block(dict[e], b)
        sharing[b, key].push(e)

fn hex(c: Char) -> U64 = U64(if c.is_digit: c - '0' else: 10 + (c - 'A'))

var flip = false
for _ in 0..m:
    let (digits, k): (Str, Int) = read()
    var word: Word = [Limb] l => U64(0)
    for l in Limb:
        for i in 0..16: word[l] = word[l] << 4 | hex(digits[l.ord * 16 + i])
        if flip: word[l] ^= ~U64(0)
    var close = false
    for b in Block:
        let key = block(word, b)
        for e in sharing[b, key]:
            if sum(popcount(word[l] ^ dict[e][l]) for l in Limb) <= k: close = true
    print(if close: 1 else: 0)
    flip = close
