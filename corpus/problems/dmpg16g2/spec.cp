// dmpg16g2: count the sub-rectangles holding all five letters, by inclusion-exclusion over the
// letters left out. Fix the bottom row; for each set of letters, a column is free while none of
// them occurs in it between the top and bottom rows. Raising the top frees columns one at a time,
// and the free runs merge in a Dsu, so each set keeps its count of all-free column spans.
let n = read<1..=500>()
domain Row = [n]
domain Col = [n]
let g = read<[Row, Col]Char>()
enum Letter: A, B, C, D, E
fn spans(k: Int) -> Int = k * (k + 1) / 2
struct FreeSpans:
    var blocked: [Col]Int
    var runs: Dsu<Col> = Dsu<Col>()
    var total = 0

    fn new(blocked: [Col]Int) -> FreeSpans:
        var s = FreeSpans(blocked: [Col] c => blocked[c] + 1)
        for c in Col:
            s.free(c)
        s

    fn free(mut self, c: Col):
        self.blocked[c] -= 1
        if self.blocked[c] > 0:
            return
        for d in [c.prev, c.next]:
            if let side = d and self.blocked[side] == 0:
                self.total -= spans(self.runs.size(side))
                self.runs.unite(side, c)
        self.total += spans(self.runs.size(c))
var last: [Col, Letter]?Row = None
var count = 0
for bot in Row:
    for c in Col:
        if let ch = Letter.at(Int(g[bot, c]) - Int('A')):
            last[c, ch] = bot
    var free = [Letter.subsets] s => FreeSpans.new([Col] c => s.items.count(ch => last[c, ch] != None))
    let leaving = last.buckets()
    for top in ..=bot:
        count += Letter.subsets.sum(of: s => if s.len % 2 == 0: free[s].total else: -free[s].total)
        for (c, ch) in leaving[Some(top)]:
            for s in Letter.subsets.filter(s => ch in s):
                free[s].free(c)
print(count)
