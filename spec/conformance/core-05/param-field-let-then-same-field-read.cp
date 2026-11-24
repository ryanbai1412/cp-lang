// expect: ok, output "3 1\n1"
// The case 8c5b0e71a moves reads only another field (`m.n`); reading `m.xs` itself at the end keeps it a copy.
struct S:
    xs: List<Int>
    n: Int
fn run(m: S) -> Int:
    var acc = m.xs
    acc.push(m.n)
    print(acc.len, m.n)
    m.xs.len
print(run(S([4, 5], 1)) - 1)
