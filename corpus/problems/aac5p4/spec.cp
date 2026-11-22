// aac5p4: written from the accepted solution (corpus/submissions/4239900.cpp).
let (n, x) = read<(1..=1_000_000_000, Int)>()
fn most(n: Int, x: Int) -> Int:
    if n == 1:
        return 0
    if n % 2 == 0:
        return n / 2
    let r = (n - 3) / 2 % 4
    n / 2 + ((x % 2 == 1) != (r == 0 or r == 3)).int
let k = most(n, x)
print(k)
print(1..=k, sep: "\n")
