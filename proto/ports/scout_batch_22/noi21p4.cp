// NOI 2021 Quantum Communication: a dictionary of n words of 256 bits
// (n <= 400000), made by the generator the statement gives, seeded with a1
// and a2. Then m questions (m <= 120000), each a word as 64 hex digits and a
// k <= 15: print 1 if some dictionary word differs from it in at most k
// bits, else 0. The questions are chained: after an answer of 1, every bit
// of the next question's word is flipped before use.
//
// Cut a word into 16 blocks of 16 bits. At most 15 differing bits leave some
// block untouched, so only the dictionary words that share a block with the
// question need comparing. The dictionary is random, so a block value has
// about n / 65536 words.
// original: corpus/submissions/3861375.cpp
use scout.bits.{U64}

domain Block = 0..16
let block_bits = 16
type Word = [Block]Int

let (n, m): (Int, Int) = read()
let (seed1, seed2): (Str, Str) = read()     // up to 2^64 - 1: too big for an Int
var (k1, k2) = (U64.parse(seed1), U64.parse(seed2))

/// The statement's generator: an xorshift on two 64-bit words, of which one bit is used.
fn next_bit() -> Int:
    var (k3, k4) = (k1, k2)
    k1 = k4
    k3 = k3.xor(k3.shl(23))
    k2 = k3.xor(k4).xor(k3.shr(17)).xor(k4.shr(26))
    k2.plus(k4).bit(32)

var dict: List<Word> = []
var sharing: [Block]Map<Int, List<Int>> = [Block] b => Map<Int, List<Int>>()   // block value -> words
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
