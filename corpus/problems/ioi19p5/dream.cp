// IOI 2019 "Vision Program": build an AND/OR/NOT circuit over an h x w image
// with exactly two black pixels that outputs whether they are at distance k.
@grader fn add_and(ins: List<Int>) -> Int
@grader fn add_or(ins: List<Int>) -> Int
@grader fn add_not(x: Int) -> Int

pub fn construct_network(h: Int, w: Int, k: Int):
    domain Cell = (r: [h], c: [w])
    fn any(ws: List<Int>) -> Int = if ws.is_empty(): add_and([0, add_not(0)]) else: add_or(ws)

    fn lines(key: Cell -> Int) -> List<Int> =
        Cell.groups(by: key).values.map(cs => add_or(cs.map(c => Int(c)))).collect()

    fn gap_exactly(ls: List<Int>, d: Int) -> Int =
        any([add_and([a, b]) for (a, b) in ls.pairs(gap: d)])

    fn gap_at_least(ls: List<Int>, d: Int) -> Int:
        let seen = ls.scan((a, b) => add_or([a, b]))
        any([add_and([b, s]) for (s, b) in seen.zip(ls.skip(d))])

    let diag = [lines(c => c.r + c.c), lines(c => c.r - c.c)]
    let hit = add_or(diag.map(ls => gap_exactly(ls, k)))
    let over = add_or(diag.map(ls => gap_at_least(ls, k + 1)))
    add_and([hit, add_not(over)])
