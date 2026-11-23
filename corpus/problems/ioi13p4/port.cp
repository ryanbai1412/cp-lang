// Source: IOI 2013 Day 2 P4, Cave, https://dmoj.ca/problem/ioi13p4
// Problem: interactive: N switches (N <= 5000), each up or down, each wired to a different
//   door of N doors in a row; a door is open iff its switch is in its hidden correct position.
//   tryCombination(S) returns the first closed door (or -1); within 70 000 calls, find every
//   switch's correct position and its door, then call answer(S, D).
// Solution: Settle doors 0, 1, ... in order, holding settled switches in their open position.
//   For door d, one query with every free switch up shows which position opens d; then binary
//   search on k, setting the first k free switches to that position and the rest to the other,
//   for the smallest k that opens d: its k-th free switch is d's. At most 1 + 13 queries per
//   door, O(N^2 log N) time.
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
        [door[w] for w in Switch if s[w] != open[w]].min()!
    fn answer(
        mut self,
        o: [Switch]Pos,
        d: [Switch]Door,
    ) -> Bool = o == open and d == door.to_array()

@graded fn explore_cave(judge: Cave):
    var open: [Switch]Pos = Up
    var door: [Switch]?Door = None
    for d in Door:
        // Known switches hold their doors open; `free` are the rest.
        let free = [w for w in Switch if door[w] == None]
        fn opens(flip: List<Switch>, to: Pos) -> Bool:
            var s = [Switch] w => if door[w] != None: open[w] elif flip.contains(w): to else: other(to)
            (judge.try_combination(s) ?? Door.K) > d
        // Which position opens door d, then which free switch carries it.
        let to = if opens(free, Up): Up else: Down
        let at = first_where(1..=free.len, k => opens(free[..k], to))
        let w = free[at - 1]
        open[w] = to
        door[w] = d
    judge.answer(open, [Switch] w => door[w]!)
