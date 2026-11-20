// APIO 2007 Zoo: n pens in a ring, one animal each. A child standing at pen
// p sees pens p..p+4 and is happy if a feared animal in view is removed or a
// loved one in view stays. Remove animals to make the most children happy.
let n: 10..=1e4 = read()
let c: 1..=5e4 = read()
domain Pen = 1..=n
domain Window = 0..5
type View = Subset<Window>                  // which of the five pens in view are emptied

var stand = Pen.first                       // generator: a child only names animals it can see
fn in_view() -> Pen: stand.cyc(rand(0, 4))

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
fn slide(v: View, empty_new: Bool) -> View: View.nth(v.idx / 2 + (if empty_new: 16 else: 0))

// Fix the view at the first pen, slide once round the ring, and come back to it.
var best = 0
for start in View:
    var cur: [View]?Int = none
    cur[start] = 0
    for p in Pen:
        var next: [View]?Int = none
        for v in View:
            if let so_far = cur[v]:
                for empty_new in [false, true]: next[slide(v, empty_new)] max= so_far + happy[p, v]
        cur = next
    best max= cur[start]!
print(best)
