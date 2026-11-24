# Friction Report: cco02p3

The v0.2 port kept its own `DIRS` table and `step` helper, held the facing as an `Int` set by
four `if facing == ...` lines, and turned with `% 4` arithmetic, each marked with a WISH. Now
the facing is a std `Dir4` from `Dir4.from("NESW", facing)!`, turns are `face.cw` and
`face.ccw`, and moves are `at.step(face)`. The package gains gen.py (boards whose free cells
are all reachable), an interactor that drives the robot and checks the map, and tests.toml; the
accepted C++ passes the same interactor.

## The way back from a recursive step is written out
- **Where:** `right(); right(); forward(); left()` after each `explore()`.
- **Ideal:** a scoped move, `with forward(): explore()`, that undoes the step on exit.

## A typed `Dir4` read
- **Where:** `let facing: Char = read()` then `Dir4.from("NESW", facing)!`.
- **Ideal:** `var face: Dir4 = read(letters: "NESW")`.

## Dream
```cp
var face: Dir4 = read(letters: "NESW")
fn explore():
    open[at] = true
    for _ in 0..4:
        if let q = at.step(face) and not open[q] and not wall[q]:
            with forward() as moved: if moved: explore() else: wall[q] = true
        right()
```
