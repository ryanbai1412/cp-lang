//! A step function over the integers, built from range additions.

/// `add(r, x)` adds `x` on every integer of `r`; `runs(clip)` then walks the result
/// as maximal runs of equal value. This is the "sort the +1 / -1 events and sweep"
/// pattern with the events, the sort and the off-by-one of the closing end kept inside.
/// `add` is O(1): the events go on a list that is sorted only when `runs` next reads it, so
/// n additions and one `runs` cost one O(n log n) sort. Coordinates are plain Ints of any
/// size: nothing is allocated per coordinate.
pub struct Steps:
    /// (coordinate, how much the value changes there), sorted only when `runs` reads them
    change: List<(Int, Int)>
    sorted_upto: Int

    pub fn new() -> Steps = Steps([], 0)

    /// Add `x` on every integer of `r`. An empty range adds nothing. O(1).
    pub fn add(mut self, r: Range<Int>, x: Int):
        if r.len == 0 or x == 0: return
        change.push((r.lo, x))
        change.push((r.hi, -x))

    /// The maximal runs of equal value inside `clip`, left to right, covering all of it
    /// (stretches nothing was added to have value 0).
    pub fn runs(mut self, clip: Range<Int>) -> List<(span: Range<Int>, value: Int)>:
        var out: List<(span: Range<Int>, value: Int)> = []
        if clip.len == 0: return out
        if sorted_upto < change.len:
            sort(mut change)
            sorted_upto = change.len
        var (cur, at) = (0, clip.lo)
        for (x, d) in change:
            if x >= clip.hi: break
            if x > at:
                out.push((span: at..x, value: cur))
                at = x
            cur += d
        out.push((span: at..clip.hi, value: cur))
        // join neighbours that ended up equal (additions that cancel)
        var joined: List<(span: Range<Int>, value: Int)> = []
        for run in out:
            if let last = joined.last, last.value == run.value:
                joined.last! = (span: joined.last!.span.lo..run.span.hi, value: run.value)
            else:
                joined.push(run)
        joined

@test
fn runs_cover_the_clip_in_order():
    var s = Steps.new()
    s.add(2..=5, 1)
    s.add(4..=9, 2)
    s.add(-3..0, 7)
    let got = [(r.span.lo, r.span.len, r.value) for r in s.runs(0..=10)]
    assert got == [(0, 2, 0), (2, 2, 1), (4, 2, 3), (6, 4, 2), (10, 1, 0)]
    assert s.runs(5..5).len == 0

@test
fn runs_agree_with_adding_cell_by_cell():
    for _ in 0..100:
        var s = Steps.new()
        var cell = [0 for _ in 0..30]
        for _ in 0..rand(0, 8):
            let (l, r, x) = (rand(-5, 34), rand(-5, 34), rand(-2, 2))
            s.add(l..=r, x)
            for i in max(l, 0)..=min(r, 29): cell[i] += x
        let (lo, hi) = (rand(0, 29), rand(0, 29))
        var at = lo
        var last: ?Int = none
        for run in s.runs(lo..=hi):
            assert run.span.lo == at and run.span.len > 0 and last != run.value
            for i in run.span: assert cell[i] == run.value
            at = run.span.hi
            last = run.value
        assert at == max(lo, hi + 1)
