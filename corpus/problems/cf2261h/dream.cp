// Codeforces 2261 H: a Hamming code on a complete binary tree. The tree holds
// 2^h - 1 bits at positions 0..size; positions that are powers of two carry
// parity (bit z of a position says which parities cover it), so the parity
// failures spell the position of one flipped bit. Every other position
// carries data, filled from the high end. Flipping node p also swaps its two
// subtrees, and `untwist` undoes that: below a 1 the children are read in
// swapped order.
// copies: 1
let op: Str = read()

fn size(h: Int) -> Int = (1 << h) - 1

// Heap numbering: node i has children 2i and 2i + 1 and sits at s[i - 1].
fn untwist(h: Int, s: Str) -> Str:
    var from: [1..=size(h)]Int = 1  // from[i]: the node of `s` that lands on node i
    var out = s
    for i in 1..=size(h):
        out[i - 1] = s[from[i] - 1]
        if 2 * i > size(h): continue
        let (kept, swapped) = (2 * from[i], 2 * from[i] + 1)
        (from[2 * i], from[2 * i + 1]) = if out[i - 1] == '1': (swapped, kept) else: (kept, swapped)
    out

fn data_slots(h: Int) -> List<Int> = [i for i in (0..size(h)).rev if i == 0 or i & (i - 1) != 0]
fn parity(s: Str, bit: Int) -> Bool = count(s[j] == '1' for j in 0..s.len if j & bit != 0) % 2 == 1

fn encode(h: Int, b: Str) -> Str:
    var s = "0".repeat(size(h))
    for (k, i) in data_slots(h).items if k < b.len: s[i] = b[k]
    for z in 0..h if parity(s, 1 << z): s[1 << z] = '1'
    untwist(h, s)

fn decode(n: Int, h: Int, sent: Str) -> Str:
    var s = untwist(h, sent)
    let bad = sum(1 << z for z in 0..h if parity(s, 1 << z))
    s[bad] = if s[bad] == '1': '0' else: '1'
    let data = Str(s[i] for i in data_slots(h))
    data[0..n].to_str()

@!multitest
let (n, h): (Int, 2..=20) = read()
let msg: Str = read()
print(if op == "first": encode(h, msg) else: decode(n, h, msg))
