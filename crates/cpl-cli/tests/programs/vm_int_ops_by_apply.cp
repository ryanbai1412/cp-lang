// Int `**`, U64 shifts and a small-tuple compare run on the VM's own ops,
// not the tree-walker.
struct P(a: Int, b: Int)
fn main():
    let n = read<Int>()
    print(n ** 3, (n + 1) ** 2 - n)
    let u: U64 = U64(n)
    let s: U64 = U64(3)
    print(u << 3, (u << 3) >> 1, u ** s, u * s - s)
    let p = P(n, 2)
    let q = P(n, n)
    print(yes(p < q), yes((n, 1) == (n, 1)), yes((n, 2) > (n, 1)))
main()
