// DMPG 2016 G2: an n×n grid of letters A–E and dots. Count subrectangles holding all five letters.
// Inclusion–exclusion over the set of letters forced absent. Fix the bottom row; as the top row
// moves down, letters whose last sighting in a column is above the top drop out, and each
// set's count of all-clear column runs only grows: a DSU of clear runs keeps Σ len·(len+1)/2.
let n: 1..=1000 = read()
domain Row = 0..n
domain Col = 0..n
let grid: [Row, Col]Char = read()
domain Letter = 0..5
domain Absent = 0..32   // a set of letters, as a mask
let holding = [Letter] l => Absent.filter(s => (s.ord >> l.ord) % 2 == 1).collect()
let sign = [Absent] s => if popcount(s.ord) % 2 == 1: -1 else: 1
fn pairs(len: Int) -> Int = len * (len + 1) / 2

var total = 0
var last: [Col, Letter]?Row = None
var seen: [Col]0..32 = 0
for bottom in Row:
    for c in Col:
        if grid[bottom, c] != '.':
            let l = Letter(Int(grid[bottom, c]) - 65)
            last[c, l] = Some(bottom)
            seen[c] |= 1 << l.ord

    // blocked[s, c]: the letters of s seen in column c between top and bottom
    var blocked: [Absent, Col]0..=5 = [Absent, Col] (s, c) => popcount(s.ord & seen[c])
    var runs = [Absent] _ => Dsu<Col>()
    var clear = [Absent] _ => 0
    for s in Absent:
        var run = 0
        for c in Col:
            if blocked[s, c] > 0:
                clear[s] += pairs(run)
                run = 0
                continue
            run += 1
            if let d = c.prev:
                if blocked[s, d] == 0: runs[s].unite(d, c)
        clear[s] += pairs(run)

    var leaving: [Row]List<(c: Col, l: Letter)> = []
    for (c, l) in (Col, Letter):
        if let r = last[c, l]: leaving[r].push((c, l))
    for top in ..=bottom:
        total += Absent.sum(of: s => sign[s] * clear[s])
        for (c, l) in leaving[top]:
            for s in holding[l]:
                blocked[s, c] -= 1
                if blocked[s, c] > 0: continue
                for d in [c.prev, c.next]:
                    if let d = d:
                        if blocked[s, d] == 0 and not runs[s].same(c, d):
                            clear[s] -= pairs(runs[s].size(d))
                            runs[s].unite(c, d)
                clear[s] += pairs(runs[s].size(c))
print(total)
