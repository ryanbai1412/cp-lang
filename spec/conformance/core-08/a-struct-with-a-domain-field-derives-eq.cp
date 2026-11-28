// expect: ok, output "YES NO\nYES"
// rule: 08.1.6
// A struct whose fields all have `Eq` derives it, a domain field too: a domain's `Eq` is its
// own row. `impl Ord` writing only `lt` serves with the derived `Eq` (R08.1.1).
domain W = 1..=3
struct R(w: W, s: Int)
impl Ord for R:
    fn lt(a, b) = a.s < b.s
struct P(w: W)
for i in W:
    if i.ord == 0:
        let r = R(i, 2)
        print(yes(r == r), yes(r < r))
        print(yes(P(i) == P(i)))
