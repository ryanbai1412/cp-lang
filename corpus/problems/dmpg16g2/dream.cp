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
    blocked: [Col]Int
    runs: Dsu<Col>
    total: Int

    fn new(blocked: [Col]Int) -> FreeSpans:
        var s = FreeSpans(blocked: [Col] c => 1, runs: Dsu<Col>(), total: 0)
        for c in Col:
            if blocked[c] == 0:
                s.free(c)
        s.blocked = blocked
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
var last: [Col, Letter]?Row = none
var count = 0
for bot in Row:
    for c in Col:
        if let ch = Letter.at(Int(g[bot, c]) - Int('A')):
            last[c, ch] = bot
    var free: [Letter.subsets]FreeSpans = [Letter.subsets] s =>
        FreeSpans.new([Col] c => s.items.count(ch => last[c, ch] != none))
    var leaves: [Row]List<(Col, Letter)> = []
    for c in Col:
        for ch in Letter:
            if let r = last[c, ch]:
                leaves[r].push((c, ch))
    for top in ..=bot:
        for s in Letter.subsets:
            count += if s.len % 2 == 0: free[s].total else: -free[s].total
        for (c, ch) in leaves[top]:
            for s in Letter.subsets:
                if ch in s:
                    free[s].free(c)
print(count)
