// IOI 2019 "Vision Program": build an AND/OR/NOT circuit over an h x w image
// with exactly two black pixels that outputs whether they are at distance k.
// Two pixels are at distance exactly k when some pair of diagonals k apart
// (in either direction) both hold a black pixel and no pair k + 1 apart does.
extern fn add_and(ins: List<Int>) -> Int
extern fn add_or(ins: List<Int>) -> Int
extern fn add_not(x: Int) -> Int

@grader
fn construct_network(h: Int, w: Int, k: Int):
    domain R = [h]
    domain C = [w]
    fn wire(r: R, c: C) -> Int = Int(r) * w + Int(c)
    fn any(ws: List<Int>) -> Int = if ws.is_empty: add_and([0, add_not(0)]) else: add_or(ws)

    fn lines(key: (R, C) -> Int) -> List<Int>:
        let cells = [(r, c) for r in R for c in C]
        let by_line = cells.group(by: rc => key(rc.0, rc.1))
        [
            add_or(
                by_line[d].map(rc => wire(rc.0, rc.1)).collect()
            ) for d in by_line.keys.collect().sorted()
        ]

    fn gap_exactly(ls: List<Int>, d: Int) -> Int =
        any([add_and([ls[i], ls[i + d]]) for i in 0..ls.len - d])

    fn gap_at_least(ls: List<Int>, d: Int) -> Int:
        let seen = ls.scan((a, b) => add_or([a, b]))
        any([add_and([seen.get(i), ls[i + d]]) for i in 0..ls.len - d])

    let diag = [lines((r, c) => Int(r) + Int(c)), lines((r, c) => Int(r) - Int(c))]
    let hit = add_or(diag.map(ls => gap_exactly(ls, k)).collect())
    let over = add_or(diag.map(ls => gap_at_least(ls, k + 1)).collect())
    add_and([hit, add_not(over)])
