// IOI 2013 Cave: n switches, each up or down, each wired to one of n doors
// in a row. try_combination(s) returns the first door still closed (or none).
// Find every switch's open position and its door in at most 70 000 tries.
//
// IDEAL SKETCH, not run: the prototype has no `protocol` / `@interactor`.
// original: corpus/submissions/4687973.cpp
let n: 1..=5000 = param()
domain Switch = 0..n
domain Door = 0..n
enum Pos: Up, Down
fn other(p: Pos) -> Pos = if p == Up: Down else: Up

protocol Cave:
    fn try_combination(s: [Switch]Pos) -> ?Door
    @final fn answer(open: [Switch]Pos, door: [Switch]Door)
    limit try_combination: 70000

// The wiring is a bijection, which no field type says. `Perm<Switch, Door>`
// is the stdlib's constructive generator for it, so nothing is rejected.
@interactor(Cave)
@derive(Gen)
struct Judge(open: [Switch]Pos, door: Perm<Switch, Door>):
    fn try_combination(mut self, s: [Switch]Pos) -> ?Door =
        min(door[w] for w in Switch if s[w] != open[w])
    fn answer(
        mut self,
        o: [Switch]Pos,
        d: [Switch]Door,
    ) -> Bool = o == open and d == door.to_array()

@graded fn explore_cave(judge: Cave):
    var open: [Switch]Pos = Up
    var door: [Switch]?Door = none
    for d in Door:
        // Known switches hold their doors open; `free` are the rest.
        let free = [w for w in Switch if door[w] == none]
        fn opens(flip: List<Switch>, to: Pos) -> Bool:
            var s = [Switch] w => if door[w] != none: open[w] elif flip.contains(w): to else: other(to)
            (judge.try_combination(s) ?? Door.K) > d
        // Which position opens door d, then which free switch carries it.
        let to = if opens(free, Up): Up else: Down
        let at = first_where(1..=free.len, k => opens(free[..k], to))
        let w = free[at - 1]
        open[w] = to
        door[w] = d
    judge.answer(open, [Switch] w => door[w]!)
