/// The sum of everything left of the boundary.
@memo
fn ways(n: Int) -> Mod<1000000007>:
    if n <= 1:
        return 1
    ways(n - 1) + ways(n - 2)

fn main():
    let (n, k): (Int, Int) = read()
    var best = 0x7f
    for i in 0..n:
        best = max(best, k * i)  // no overflow: the types say so
    print("{best} {ways(n)}", 'x')
