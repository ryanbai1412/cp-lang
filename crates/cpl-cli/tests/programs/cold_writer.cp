// What a first draft reaches for: a one-line enum after `:`, a number's
// math as a member, and `flush()`.
enum Node: Leaf | Inner
let x: F64 = read()
let n: Int = read()
print(x.sqrt, x.floor, x.ceil, x.round, x.trunc, (-x).abs)
print(n.sqrt, n.isqrt, (-n).abs)
flush()
print(Node.Leaf == Node.Leaf, Node.Leaf == Node.Inner)
const MOD = 998244353
type Z = Mod<MOD>
let k: 1..=10 = read()
let zs: [k]Z = read()
let three: Z = 3
print(three ** 5, sum(zs), sum(z * z for z in zs))
