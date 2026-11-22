// CSES 1652 Forest Queries: count '*' in q rectangles of an n x n grid.
let (n, q): (1..=1000, 1..=2e5) = read()
domain Row = 1..=n
domain Col = 1..=n
let forest = read<[Row, Col]Char>()
let trees = Prefix<Row, Col, Int>([Row, Col] (r, c) => if forest[r, c] == '*': 1 else: 0)
for _ in 0..q:
    let (y1, x1, y2, x2) = read<(Row, Col, Row, Col)>()
    print(trees.fold(y1..=y2, x1..=x2))
