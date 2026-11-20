// IOI 2021 Day 1 P1, Distributing Candies
// original: corpus/submissions/4698803.cpp

// What a stretch of days does to an unbounded box starting at 0: where the
// count ends, and the highest and lowest counts it touches (the start included).
struct Walk:
    end: Int = 0
    top: Int = 0
    low: Int = 0
    fn then(self, o: Walk) -> Walk: Walk(self.end + o.end, max(self.top, self.end + o.top), min(self.low, self.end + o.low))
    fn span(self) -> Int: self.top - self.low

fn step(v: Int) -> Walk: Walk(v, max(v, 0), min(v, 0))

@grader
fn distribute_candies(cap: [Box]Int, l: [Day]Box, r: [Day]Box, v: [Day]Int) -> [Box]Int:
    domain Box, Day                              // inferred from the argument shapes
    // Sweep the boxes keeping, per day, the change that day makes to the current box.
    var starts: [Box]List<Day> = []
    var stops: [Box.bounds]List<Day> = []
    for d in Day:
        starts[l[d]].push(d)
        stops[r[d].r].push(d)
    var days = SegTree<Day, Walk>(Walk(), (a, b) => a.then(b))
    var ans: [Box]Int = 0
    for box in Box:
        for d in starts[box]: days.set(d, step(v[d]))
        for d in stops[box.l]: days.set(d, Walk())
        // The days from `tail` on never span the whole box. The day before
        // them (if any) began a move of a full box, so the count was pinned
        // to a wall at the tail's highest or lowest point and is free after.
        let tail = days.min_left(Day.last.r, w => w.span() < cap[box])
        let after = days.fold(tail..)
        ans[box] = if let p = tail.left and v[p] > 0: cap[box] - (after.top - after.end)
                   else: after.end - after.low
    ans
