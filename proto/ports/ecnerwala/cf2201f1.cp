// Codeforces 2201 F1: check whether the transposed Ferrers diagrams of the two
// row-frequency multisets match after each update via a double modular hash.
const M1 = 998244353
const M2 = 1_000_000_007
type Z1 = Mod<M1>
type Z2 = Mod<M2>

fn mix1(f: Int, g: Int) -> Z1:
    let x: Z1 = f + 1315423911
    let y: Z1 = g + 2654435761
    let t = x * x + y * y * 31337 + x * y * 1000003 + 7
    t * t * t

fn mix2(f: Int, g: Int) -> Z2:
    let x: Z2 = f + 19260817
    let y: Z2 = g + 1000000009
    let t = x * x * 998244353 + y * y + x * y * 19260817 + 13
    t * t * t

@!multitest
let (n, q): (Int, Int) = read()
var row_freq: [0..2, 0..n]Int = 0
var freq_freq: [0..2, 0..n]Int = 0
var h1: [0..2]Z1 = 0
var h2: [0..2]Z2 = 0

for _ in 0..q:
    let (r0, r1): (Int, Int) = read()
    let rs: [0..2]Int = [r0 - 1, r1 - 1]
    for z in 0..2:
        let r = rs[z]
        let f = row_freq[z, r]
        row_freq[z, r] = f + 1
        let g = freq_freq[z, f]
        freq_freq[z, f] = g + 1
        let a = if z == 0: f else: g
        let b = if z == 0: g else: f
        h1[z] += mix1(a, b)
        h2[z] += mix2(a, b)
    print(if h1[0] == h1[1] and h2[0] == h2[1]: "YES" else: "NO")
