// expect: ok, output "7"
// rule: 01.9.1 03.1.3
fn f(n: 1..=10) -> Int:
    domain V = 0..n
    struct Pt: v: V, w: 0..=1e9
    let p = Pt(V(0), 4)
    p.w + V.count
print(f(3))
