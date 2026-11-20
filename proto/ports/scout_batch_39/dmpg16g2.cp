// DMPG '16 G2 — original: corpus/submissions/4653887.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let n: 1..=500 = read()
domain Row = 0..n
domain Col = 0..n
let g: [Row, Col]Char = read()

enum Letter = A | B | C | D | E
type Mask = Subset<Letter>

fn c2(x: Int) -> Int:
    x * (x + 1) / 2

struct ZeroRuns:
    blocked: [Col]Int
    dsu: Dsu<Col>
    total: Int

    fn new(init: [Col]Int) -> ZeroRuns:
        var dsu = Dsu<Col>()
        var total = 0
        var run = 0
        for c in Col:
            if init[c] == 0:
                run += 1
                if let prev = c - 1 if init[prev] == 0:
                    dsu.unite(prev, c)
            else:
                total += c2(run)
                run = 0
        total += c2(run)
        ZeroRuns(blocked: init, dsu: dsu, total: total)

    fn erase(mut self, c: Col):
        self.blocked[c] -= 1
        if self.blocked[c] > 0:
            return
        if let prev = c - 1 if self.blocked[prev] == 0:
            self.total -= c2(self.dsu.size(prev))
            self.dsu.unite(prev, c)
        if let next = c + 1 if self.blocked[next] == 0:
            self.total -= c2(self.dsu.size(next))
            self.dsu.unite(c, next)
        self.total += c2(self.dsu.size(c))

var last: [Col, Letter]?Row = none
var ans = 0

for bot in Row:
    for c in Col if g[bot, c] != '.':
        // FRICTION 39.9: @tags char-range cannot bind directly to enum Letter payload
        last[c, Letter.from_idx(g[bot, c] - 'A')!] = bot

    // FRICTION 39.9: iterating all subsets of Mask and mutating structs inside [Mask]ZeroRuns
    var ds: [Mask]ZeroRuns = [Mask] mask =>
        let init = [Col] c => count(ch in mask and last[c, ch] != none for ch in Letter)
        ZeroRuns.new(init)

    var events: [Row]List<(Col, Letter)> = [Row] _ => []
    for c in Col:
        for ch in Letter:
            if let r = last[c, ch]:
                events[r].push((c, ch))

    for top in ..=bot:
        for mask in Mask:
            if mask.len % 2 == 1: ans -= ds[mask].total
            else: ans += ds[mask].total
        for (c, ch) in events[top]:
            for mask in Mask if ch in mask:
                ds[mask].erase(c)

print(ans)
