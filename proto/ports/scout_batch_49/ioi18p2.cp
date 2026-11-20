// IOI 2018 Day 1 P2, Seats
// original: corpus/submissions/4731161.cpp
type MinCount = (least: Int, times: Int)
fn both(a: MinCount, b: MinCount) -> MinCount:
    if a.least != b.least: min(a, b) else: (a.least, a.times + b.times)

@grader
struct Hall:
    domain Row, Col, Who             // FRICTION: fixed by give_initial_chart, needed by every field
    type Cell = (Row, Col)
    seat: [Who]Cell
    who: [Row, Col]Who
    // Position t stands for the prefix ..=t. It counts the prefix's top-left
    // corners (a seated cell with nobody above or to the left) plus its dents
    // (an empty cell with two or more seated neighbours). A prefix is a
    // rectangle exactly when that total is 1, the least it can be.
    bad: LazySegTree<Who, MinCount>

    fn give_initial_chart(h: Int, w: Int, r: [Who]Row, c: [Who]Col) -> Hall:
        domain Row = 0..h
        domain Col = 0..w
        domain Who = 0..h * w
        var who: [Row, Col]Who = Who.first
        for p in Who: who[r[p], c[p]] = p
        // FRICTION: "add on a range, count the minima" is not a stock tag
        let add = Action(0, +, (d, x) => (x.least + d, x.times))
        var hall = Hall([Who] p => (r[p], c[p]), who,
                        LazySegTree<Who, MinCount>((Who.K, 0), both, add, [Who] _ => (0, 1)))
        for cell in Cell: hall.account(cell, 1)
        hall

    // when the contestant at q arrives; none when q is off the grid
    fn arrival(self, q: ?Cell) -> ?Who: self.who[q?]

    // adds d over the times `cell` is a corner, and over the times it is a dent
    fn account(mut self, cell: Cell, d: Int):
        let me = self.who[cell]
        // a corner from my arrival until the first of my upper and left neighbours
        if let covered = min(self.arrival(cell.step(Dir4.N)), self.arrival(cell.step(Dir4.W))):
            self.bad.apply(me..covered, d)
        else:
            self.bad.apply(me.., d)
        // a dent from my second neighbour's arrival until my own
        let near = sorted(self.who[q] for q in cell.adj4)
        if near.len >= 2: self.bad.apply(near[1]..me, d)

    fn swap_seats(mut self, a: Who, b: Who) -> Int:
        // the two seats and their neighbours are the only cells whose spans change
        var cells = Set<Cell>()
        for p in [a, b]:
            cells.add(self.seat[p])
            for q in self.seat[p].adj4: cells.add(q)
        for q in cells: self.account(q, -1)
        swap(mut self.seat[a], mut self.seat[b])
        self.who[self.seat[a]] = a
        self.who[self.seat[b]] = b
        for q in cells: self.account(q, 1)
        self.bad.fold(..).times
