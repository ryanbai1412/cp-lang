// Codeforces 2261 H: a Hamming code on a complete binary tree. The tree holds
// 2^h - 1 bits; positions that are powers of two carry parity, so one flipped
// bit can be located. Flipping node p also swaps its two subtrees, and
// `untwist` undoes that: below a 1 the children are read in swapped order.
let op: Str = read()

fn untwist(h: Int, s: Str) -> Str:
    let size = (1 << h) - 1
    var from: [1..=size]Int = 1  // from[i]: the node of `s` that lands on node i
    var out = s
    for i in 1..=size:
        out[i - 1] = s[from[i] - 1]
        if 2 * i > size: continue
        let f = Int(out[i - 1] == '1')
        from[2 * i] = 2 * from[i] + f
        from[2 * i + 1] = 2 * from[i] + 1 - f
    out

fn is_data(i: Int) -> Bool = i == 0 or i & (i - 1) != 0
fn parity(s: Str, bit: Int) -> Bool = count(s[j] == '1' for j in 0..s.len if j & bit != 0) % 2 == 1

fn encode(h: Int, b: Str) -> Str:
    let size = (1 << h) - 1
    var s = "0".repeat(size)
    var at = 0
    for i in (0..size).rev:
        if is_data(i):
            if at < b.len: s[i] = b[at]
            at += 1
        elif parity(s, i): s[i] = '1'
    untwist(h, s)

fn decode(n: Int, h: Int, sent: Str) -> Str:
    var s = untwist(h, sent)
    var bad = 0
    for z in 0..h if parity(s, 1 << z): bad += 1 << z
    s[bad] = if s[bad] == '1': '0' else: '1'
    Str(s[i] for i in (0..s.len).rev if is_data(i))[0..n].to_str()

@!multitest
let (n, h): (Int, 2..=20) = read()
let msg: Str = read()
print(if op == "first": encode(h, msg) else: decode(n, h, msg))
