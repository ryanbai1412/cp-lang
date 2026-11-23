// ioi18p1 Combo: find a hidden word over A B X Y whose first letter never comes back, in n + 2
// presses. Two presses find the first letter; then one press settles each next letter: with the
// other three letters a, b, c, the press "s a | s b a | s b b | s b c" grows by 1 for a, 2 for b
// and 0 for c. The last letter takes two plain presses.
extern fn press(p: Str) -> Int
@grader
fn guess_sequence(n: Int) -> Str:
    let first = if press("AB") >= 1: (if press("A") == 1: 'A' else: 'B') else: (
        if press("X") == 1: 'X' else: 'Y'
    )
    let o = "ABXY".filter(ch => ch != first).collect()
    var s = Str.of([first])
    for _ in 2..n:
        let grew = press(
            s + [o[0]] + s + [o[1], o[0]] + s + [o[1], o[1]] + s + [o[1], o[2]]
        ) - s.len
        s.push(if grew == 1: o[0] else: (if grew == 2: o[1] else: o[2]))
    if n >= 2:
        s.push(if press(s + [o[0]]) == n: o[0] else: (if press(s + [o[1]]) == n: o[1] else: o[2]))
    s
