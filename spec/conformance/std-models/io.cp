// expect: ok, output "ok"
// input: "1 300\n"
// model: yes
// model: Wire
// model: read_all
// `read_all` takes the whole `seed n` input, a `Wire` puts the run in interactive mode, then
// seeded random `yes` calls (default words, `answers:`, and a `with io(answers:)` scope) are
// checked against the plain `if`. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let wire = Wire()
let input = read_all<Int>()
var bad = if input.len == 2: "" else: "read_all: {input.len} values, want 2"
let seed0 = input[0]
let n = input[1]
var rng = Rng(seed0)
let words = [("Yes", "No"), ("Possible", "Impossible"), ("A", "B")]
for k in 0..n:
    if bad != "": break
    let ok = rng.chance(0.5)
    let (t, f) = words[rng.int(0, 2)]
    if yes(ok) != (if ok: "YES" else: "NO"): bad = "op {k}: yes"
    elif yes(ok, answers: Some((t, f))) != (if ok: t else: f): bad = "op {k}: yes(answers: {t}/{f})"
    else:
        with io(answers: ("Yes", "No")):
            if yes(ok) != (if ok: "Yes" else: "No"): bad = "op {k}: yes under io(answers:)"
print(if bad == "": "ok" else: bad)
