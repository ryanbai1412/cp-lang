let (n, q) = read<(1..=1000, 1..=2e5)>()
domain R = 1..=n
domain C = 1..=n
let g = read<[R, C]Char>()
let trees = [R, C] (r, c) => if g[r, c] == '*': 1 else: 0
let p = Prefix<R, C, Int>(trees)
for _ in 0..q:
    let (r1, c1, r2, c2) = read<(R, C, R, C)>()
    print(p.fold(r1..=r2, c1..=c2))
