// IOI 2018 Combo: find a hidden string over ABXY whose first letter never repeats, with
// `press(p)` returning the longest prefix of the secret that is a substring of p.
// Two presses find the first letter; each middle letter costs one press of four candidates;
// the last costs two.
let n: 1..=2000 = read()
fn press(p: Str) -> Int = ask("press", p)

fn guess_sequence(n: Int) -> Str:
    fn among(s: Str, two: List<Char>) -> Char:
        if press(s + two[0]) > s.len: two[0] else: two[1]
    let first = among("", if press("AB") > 0: ['A', 'B'] else: ['X', 'Y'])
    let rest = "ABXY".filter(c => c != first).collect()
    let (p, q, r) = (rest[0], rest[1], rest[2])
    var s = Str([first])
    while s.len < n - 1:
        let got = press(s + p + s + q + p + s + q + q + s + q + r) - s.len
        s.push(match got: 0 => r, 1 => p, _ => q)
    if s.len < n:
        let last = if press(s + p + s + q) > s.len: [p, q] else: [r, r]
        s.push(among(s, last))
    s

print("guess", guess_sequence(n))
