// Source: NOI 2021 P4 (Quantum Communication), https://dmoj.ca/problem/noi21p4
// limits: 3000 ms, 384 MB
// Problem: a dictionary of n <= 4e5 random 256-bit words comes from the statement's generator,
//   seeded with a1, a2. Then m <= 1.2e5 queries, each 64 hex digits and k <= 15: print 1 if a
//   dictionary word differs from the query in at most k bits, else 0. Online: after an answer
//   of 1, every bit of the next query is flipped.
// Solution: cut words into 16 blocks of 16 bits; at most 15 differing bits leave some block
//   equal (pigeonhole). Per block, the words with each block value; a query compares
//   only the words sharing one of its blocks, by popcount of the block xors. Random words put
//   about n / 65536 in a bucket: ~100 candidates of 16 popcounts per query.
// Bounds: n: 1..=4e5; m: 1..=1.2e5; k: 0..=15
// original: corpus/submissions/3861375.cpp
domain Block = 0..16
domain Half = 0..1 << 16  // the value of one 16-bit block
type Word = [Block]Half

let (n, m): (1..=4e5, 1..=1.2e5) = read()
domain W = 0..n
var (k1, k2): (U64, U64) = read()  // up to 2^64 - 1: too big for an Int

/// The statement's generator: an xorshift on two 64-bit words, of which one bit is used.
fn next_bit() -> Int:
    var (k3, k4) = (k1, k2)
    k1 = k4
    k3 ^= k3 << 23
    k2 = k3 ^ k4 ^ (k3 >> 17) ^ (k4 >> 26)
    Int((k2 + k4) >> 32 & U64(1))

fn next_half() -> Half:
    var v = 0
    repeat 16: v = v << 1 | next_bit()
    Half(v)

let dict: [W]Word = [W] _ => [Block] _ => next_half()
var sharing: [Block, Half]List<W> = []  // the words with each block value
for i in W:
    for b in Block: sharing[b, dict[i][b]].push(i)

fn bits_apart(a: Word, b: Word) -> Int = sum(popcount(a[c].ord ^ b[c].ord) for c in Block)

var flip = false
repeat m:
    let (digits, k): (Str, 0..=15) = read()
    let mask = if flip: Half.last!.ord else: 0
    let word: Word = [Block] b => Half(digits[4 * b.ord..4 * b.ord + 4].parse_int(base: 16)! ^ mask)
    flip = any(bits_apart(word, dict[i]) <= k for b in Block for i in sharing[b, word[b]])
    print(if flip: 1 else: 0)
