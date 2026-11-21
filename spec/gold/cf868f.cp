// Cut a into k pieces, minimising pairs of equal elements inside a piece.
// The best last cut moves right as the end moves right, so each layer is divide and conquer
// over ends. cost(c..i) comes from one window whose ends slide: O(n log n) slides per layer.
use std.window.{Window}

let n: 2..=100_000 = read()
let k: 2..=20 = read()
domain Pos = 0..n
domain Cut = Pos.cuts           // a range of cuts c..i is the piece a[c..i]
domain Value = 1..=n
let a: [Pos]Value = read()

var inside: [Value]Int = 0
var pairs = 0
fn enter(x: Pos):
    pairs += inside[a[x]]
    inside[a[x]] += 1
fn leave(x: Pos):
    inside[a[x]] -= 1
    pairs -= inside[a[x]]
var window = Window<Pos>(enter, leave)

fn cost(piece: Range<Pos>) -> Int:   // equal pairs inside the piece
    window.move(to: piece)
    pairs

fn layer(prev: [Cut]Int) -> [Cut]Int:
    var next: [Cut]Int = 0
    fn fill(ends: Range<Cut>, cuts: Range<Cut>):   // next over ends; their best cuts lie in cuts
        let (left, i, right) = ends.halve() ?? return
        let (v, c) = min((prev[j] + cost(j..i), j) for j in cuts.clip(..=i))
        next[i] = v
        fill(left, cuts.clip(..=c))
        fill(right, cuts.clip(c..))
    fill(Cut.all, Cut.all)
    next

var best: [Cut]Int = [Cut] i => cost(..i)   // empty pieces cost 0, so "at most k" equals "exactly k"
repeat k - 1: best = layer(best)
print(best[Cut.last])
