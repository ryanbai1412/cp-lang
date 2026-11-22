// p03 with a hand-built 2-D prefix table (rows 1-2 of LOG-v03.md).
let (n, q): (1..=1000, 1..=2e5) = read()
domain Row = 1..=n
domain Col = 1..=n
let forest = read<[Row, Col]Char>()
var pre: [0..=n, 0..=n]Int = 0
for r in Row:
    for c in Col:
        let (i, j) = (Int(r), Int(c))
        pre[i, j] = pre[i - 1, j] + pre[i, j - 1] - pre[i - 1, j - 1] + (if forest[r, c] == '*': 1 else: 0)
for _ in 0..q:
    let (y1, x1, y2, x2) = read<(1..=n, 1..=n, 1..=n, 1..=n)>()
    print(pre[y2, x2] - pre[y1 - 1, x2] - pre[y2, x1 - 1] + pre[y1 - 1, x1 - 1])
