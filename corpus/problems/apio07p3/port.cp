// Source: APIO 2007 P3 (Zoo), https://dmoj.ca/problem/apio07p3
// limits: 2000 ms, 16 MB
// Problem: n <= 1e4 enclosures in a circle, c <= 5e4 children; each sees five consecutive ones
//   E..E+4 (wrapping) and names some there it fears and some it loves. A child is happy if a
//   feared animal is removed or a loved one stays. Choose animals to remove; print the most
//   children that can be made happy.
// Solution: a child only depends on which of its 5 pens are emptied, so tabulate happy[p, mask]
//   over the 32 masks of the window starting at p. DP round the ring with the window's mask as
//   state: fix the first window's mask, slide one pen at a time (drop the old pen, decide the
//   new one) adding happy[p, mask], and end on the starting mask. O(32 * 32 * 2 * n).
let n: 10..=1e4 = read()
let c: 1..=5e4 = read()
domain Pen = 1..=n
domain Window = 0..5
type View = Subset<Window>  // which of the five pens in view are emptied

var stand = Pen.first!  // generator: a child only names animals it can see
fn in_view() -> Pen: Pen.wrap(stand.ord + rand(0, 4))

// happy[p, gone]: children at pen p who are happy when `gone` is emptied in their view
var happy: [Pen, View]Int = 0
for _ in 0..c:
    let pen: Pen = read()
    stand = pen
    let fears: 0..=5 = read()
    let loves: 0..=5 = read()
    var fear = View.empty
    for _ in 0..fears:
        let x: Pen = read(gen: in_view)
        fear = fear.with(Window.nth((x - pen) % n))
    var love = View.empty
    for _ in 0..loves:
        let x: Pen = read(gen: in_view)
        love = love.with(Window.nth((x - pen) % n))
    for gone in View:
        if not (gone & fear).is_empty or not (love - gone).is_empty: happy[pen, gone] += 1

// the view one pen further on: drop the pen left behind, decide the new one
fn slide(v: View, empty_new: Bool) -> View:
    var kept = View.empty
    for w in Window:
        if let down = w.prev and v.with(w) == v: kept = kept.with(down)  // `v.contains(w)` once `cpl run` has it
    if empty_new: kept.with(Window.last!) else: kept

// Fix the view at the first pen, slide once round the ring, and come back to it.
var best = 0
for start in View:
    var cur: [View]?Int = None
    cur[start] = 0
    for p in Pen:
        var next: [View]?Int = None
        for v in View:
            if let so_far = cur[v]:
                for empty_new in Bool: next[slide(v, empty_new)] max= so_far + happy[p, v]
        cur = next
    best max= cur[start]!
print(best)
