// Cut a into k pieces, minimising pairs of equal elements inside a piece.
// The best last cut moves right as the end moves right, so each layer is divide and conquer
// over ends. cost(q..i) comes from one window whose ends slide: O(n log n) slides per layer.
use std.window.{Window}

let n: 2..=100_000 = read()
let k: 2..=20 = read()
domain Pos = 0..n
domain Cut = Pos.bounds           // q..i, a start position to an end cut, is the piece a[q..i]
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
    var next = prev.clone()                         // an empty last piece keeps prev
    fn fill(ends: Range<Pos>, starts: Range<Pos>):  // pieces ending at each p; best starts lie in starts
        let p = ends.mid else: return
        let (v, q) = min((prev[q.l] + cost(q..p.r), q) for q in starts.clip(..p.r))
        next[p.r] min= v
        fill(ends.l..p.l, starts.l..q.r)
        fill(p.r..ends.r, q.l..starts.r)
    fill(Pos.all, Pos.all)
    next

var best: [Cut]Int = [Cut] i => cost(..i)   // empty pieces cost 0, so "at most k" equals "exactly k"
repeat k - 1: best = layer(best)
print(best[Cut.last])
