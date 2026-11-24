# ioi13p4: Cave

The task is a grader function: the judge answers `tryCombination` and
receives `answer`. The port cannot state that protocol, so it speaks it
as text over stdio to `interactor.py` (`try S`, `answer S D`), and the
switch positions are bare `0..=1`.

## No way to declare a function-call interactive protocol
- **Where:** port lines 20-22 and 43: `try_combination` is
  `ask("try", s)` and the result is `print("answer", ...)`, a text protocol
  with `interactor.py` holding the wiring and the 70 000-try limit; N is
  `read()`, not given by the grader. The dream's `protocol Cave:` block is E0107
  "expected the end of the statement, found a name", and `param()` (N
  given by the grader, not read) is E0201 "no `param` in scope, and not in
  the stdlib (yet)".
- **What felt wrong:** the call interface becomes stringly `ask` messages;
  nothing in the program states the protocol or its try limit.
- **Goal it hurts:** express, correct (the judge stand-in can drift from the
  statement).
- **Ideal code:** the dream below: `protocol` with `limit`, an
  `@interactor` judge struct and an `@graded` entry fn.
- **General fix:** a grader-protocol declaration (functions the judge
  provides, a call limit, the final call) that `cpl build` lowers to the
  IOI-style extern functions and `cpl stress` drives with an interactor.
- **Severity:** high for function-interface tasks (every IOI grader task).

## Switch positions as `0..=1`
- **Where:** port lines 20, 27, 33: `[Switch]0..=1` and `1 - to`.
- **What felt wrong:** the dream's `enum Pos: Up | Down` with `other(p)`
  needs the grader's 0/1 encoding; with input read directly, the port keeps
  the encoding.
- **General fix:** comes with the protocol: an enum's io encoding (`Up = 0`).
- **Severity:** low.

## Dream

```cp
let n: 1..=5000 = param()
domain Switch = 0..n
domain Door = 0..n
enum Pos: Up | Down
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
            (judge.try_combination(s) ?? Door.count) > d
        // Which position opens door d, then which free switch carries it.
        let to = if opens(free, Up): Up else: Down
        let at = first_where(1..=free.len, k => opens(free[..k], to))
        let w = free[at - 1]
        open[w] = to
        door[w] = d
    judge.answer(open, [Switch] w => door[w]!)
```
