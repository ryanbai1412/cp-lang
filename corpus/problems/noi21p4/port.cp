// Source: NOI 2021 P4 (Quantum Communication), https://dmoj.ca/problem/noi21p4
// limits: 3000 ms, 384 MB
// Problem: a dictionary of n <= 4e5 random 256-bit words comes from the statement's generator,
//   seeded with a1, a2. Then m <= 1.2e5 queries, each 64 hex digits and k <= 15: print 1 if a
//   dictionary word differs from the query in at most k bits, else 0. Online: after an answer
//   of 1, every bit of the next query is flipped.
// Solution: cut words into 16 blocks of 16 bits; at most 15 differing bits leave some block
//   equal (pigeonhole). Per block, a map from block value to the words with it; a query compares
//   only the words sharing one of its blocks, by popcount of the block xors. Random words put
//   about n / 65536 in a bucket: ~100 candidates of 16 popcounts per query.
// original: corpus/submissions/3861375.cpp
domain Block = 0..16
let block_bits = 16
type Word = [Block]Int

let (n, m): (Int, Int) = read()
var (k1, k2): (U64, U64) = read()  // up to 2^64 - 1: too big for an Int

/// The statement's generator: an xorshift on two 64-bit words, of which one bit is used.
fn next_bit() -> Int:
    var (k3, k4) = (k1, k2)
    k1 = k4
    k3 ^= k3 << 23
    k2 = k3 ^ k4 ^ (k3 >> 17) ^ (k4 >> 26)
    Int((k2 + k4) >> 32 & U64(1))

var dict: List<Word> = []
var sharing: [Block]Map<Int, List<Int>> = [Block] b => Map<Int, List<Int>>()  // block value -> words
for i in 0..n:
    var word: Word = [Block] b => 0
    for b in Block:
        for _ in 0..block_bits: word[b] = word[b] << 1 | next_bit()
        if not sharing[b].contains(word[b]): sharing[b][word[b]] = []
        sharing[b][word[b]].push(i)
    dict.push(word)

fn hex(c: Char) -> Int = if c.is_digit: c - '0' else: 10 + (c - 'A')

var flip = false
for _ in 0..m:
    let (digits, k): (Str, Int) = read()
    var word: Word = [Block] b => 0
    for b in Block:
        for i in 0..4: word[b] = word[b] << 4 | hex(digits[b.ord * 4 + i])
    if flip:
        for b in Block: word[b] = word[b] ^ ((1 << block_bits) - 1)
    var close = false
    for b in Block:
        for i in sharing[b].get(word[b]) ?? []:
            if sum(popcount(word[c] ^ dict[i][c]) for c in Block) <= k: close = true
    print(if close: 1 else: 0)
    flip = close
