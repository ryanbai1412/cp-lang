// DMPG 2016 G2: an n×n grid of letters A–E and dots. Count subrectangles holding all five letters.
// Inclusion–exclusion over the set of letters forced absent. Fix the bottom row; as the top row
// moves down, letters whose last sighting in a column is above the top drop out, and each
// subset's count of all-clear column runs only grows: a DSU of clear runs keeps Σ len·(len+1)/2.
enum Letter: A = 'A', B = 'B', C = 'C', D = 'D', E = 'E'
domain R = [read Int]
domain Col = [R.len]
let grid = read [R, Col]?Letter

struct ClearRuns(blocked: [Col]Int):
    runs: Dsu<Col> = Dsu()
    pairs: Int = 0
    fn init(mut self):
        for run in self.blocked.runs(by: b => b == 0).filter(r => self.blocked[r.first!] == 0):
            self.pairs += run.len * (run.len + 1) / 2
            for (x, y) in run.keys.adjacent: self.runs.union(x, y)
    fn unblock(mut self, c: Col):
        self.blocked[c] -= 1
        if self.blocked[c] > 0: return
        self.pairs += 1
        for d in [c.prev, c.next]:
            if d is Some(d) and self.blocked[d] == 0 and not self.runs.same(c, d):
                self.pairs += self.runs.size(c) * self.runs.size(d)
                self.runs.union(c, d)

var total = 0
var last: [Col, Letter]?R = None
for bottom in R:
    for c in Col: if grid[bottom, c] is Some(l): last[c, l] = Some(bottom)
    var count = [for S in Letter.subsets: ClearRuns([for c in Col: S.count(l => last[c, l] != None)])]
    let leaving = R.group(last.items.filter_map(((c, l), r) => r.map(r => (r, (c, l)))))
    for top in ..=bottom:
        total += count.items.sum(of: (S, runs) => (-1) ** S.len * runs.pairs)
        for (c, l) in leaving[top]:
            for S in Letter.subsets.filter(S => l in S): count[S].unblock(c)
print(total)
