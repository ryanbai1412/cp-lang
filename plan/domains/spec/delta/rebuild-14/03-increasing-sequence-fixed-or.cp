// Codeforces 1988C Increasing Sequence with Fixed OR
// in: t; per case n. out: per case the length k, then the k numbers of the sequence
@!multitest
let n = read<1..=1e18>()
var seq = [n ^ (1 << b) for b in (0..60).rev if n.bit(b) and n != 1 << b]
seq.push(n)
print(seq.len)
print(seq)
