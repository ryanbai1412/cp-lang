let (n, q) = read<(1..=1000, 1..=2e5)>()
domain R = 1..=n
domain C = 1..=n
let forest = read<[R, C]Char>()
let trees = prefix([R, C] (r, c) => Int(forest[r, c] == '*'))
for query in read<[q](y1: R, x1: C, y2: R, x2: C)>():
    print(trees.fold(query.y1..=query.y2, query.x1..=query.x2))
