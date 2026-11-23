// IOI 2018 Combo: find a hidden string over ABXY whose first letter never repeats, with
// `press(p)` returning the longest prefix of the secret that is a substring of p.
// Two presses find the first letter; each middle letter costs one press of four candidates;
// the last costs two.
fn guess_sequence(n: Int) -> Str:
    fn among(s: Str, two: [Char]) -> Char:
        if press(s + two[0]) > s.len: two[0] else: two[1]
    let first = among("", if press("AB") > 0: ['A', 'B'] else: ['X', 'Y'])
    let [p, q, r] = "ABXY".filter(c => c != first)
    var s = Str(first)
    while s.len < n - 1:
        let got = press(s + p + s + q + p + s + q + q + s + q + r) - s.len
        s.push(match got: 0 => r, 1 => p, _ => q)
    if s.len < n:
        let last = if press(s + p + s + q) > s.len: [p, q] else: [r, r]
        s.push(among(s, last))
    s
