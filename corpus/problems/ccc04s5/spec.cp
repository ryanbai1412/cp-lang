// ccc04s5 Super Plumber: from the bottom-left to the bottom-right exit. In each column you
// enter, slide up or down (never through a `*`) collecting digits, then step right. Columns
// are processed right to left: best[r] is the most gold from entering this column at row r.
while true:
    let (rows, cols) = read<(0..=100, 0..=10)>()
    if rows == 0 and cols == 0:
        break
    domain R = 0..rows
    domain C = 0..cols
    let g = read<[R, C]Char>()
    var best: [R]?Int = None
    best[R.last!] = 0
    for c in C.rev:
        fn reach<S: Seq>(path: S) -> yields (k: R, gold: Int):
            var gold = 0
            for k in path.take_while(k => g[k, c] != '*'):
                gold += if g[k, c].is_digit: g[k, c].digit else: 0
                yield (k, gold)
        best = R.map(r => chain(reach(r..), reach((..=r).rev))
            .filter_map(p => best[p.k].map(x => x + p.gold))
            .max()).collect()
    print(best[R.last!]!)
