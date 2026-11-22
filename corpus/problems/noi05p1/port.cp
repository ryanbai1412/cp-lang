// NOI 2005 P1, Gorgeous Waltz: a piano stands on a ballroom floor with
// furniture (x). In each of q stretches of time the floor tilts one way
// and the piano may slide at most one cell per second that way, or be
// held still. Print the longest total slide.
// original: corpus/submissions/5032738.cpp
let n: 1..=200 = read()
let m: 1..=200 = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
let start: Cell = read()
let q: 1..=200 = read()
let hall: [Row, Col]Char = read()
domain Tilt = 1..=q
let tilts: [Tilt](from: Int, to: Int, way: 1..=4) = read()   // WISH L2: way: Dir4, numbered N S W E

// Every row or column of the hall, walked in the direction of the tilt.
fn lines(way: Int) -> List<List<Cell>>:                   // WISH L3: for line in Cell.lines(way)
    if way == 1: return [[(r, c) for r in Row.rev] for c in Col]
    if way == 2: return [[(r, c) for r in Row] for c in Col]
    if way == 3: return [[(r, c) for c in Col.rev] for r in Row]
    return [[(r, c) for c in Col] for r in Row]

var far: [Row, Col]?Int = none                            // longest slide that ends here
far[start] = 0
for (from, to, way) in tilts:
    let seconds = to - from + 1
    var next: [Row, Col]?Int = none
    for line in lines(way):
        // (slide minus place, place) of the starts still in reach, best first
        var reach: List<(Int, Int)> = []                  // WISH L15: sliding(max, xs, width) over a MonoQueue
        var head = 0
        for i in 0..line.len:
            let p = line[i]
            if hall[p] == 'x':                            // WISH L16: for run in line.split(p => hall[p] == 'x')
                reach = []
                head = 0
            else:
                if let slid = far[p]:
                    while reach.len > head and reach.last!.0 <= slid - i: reach.pop()
                    reach.push((slid - i, i))
                if head < reach.len and reach[head].1 < i - seconds: head += 1
                if head < reach.len: next[p] = reach[head].0 + i
    far = next
print(max(far[r, c] ?? 0 for r in Row for c in Col))
