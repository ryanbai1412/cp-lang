// expect: ok, output "4"
// rule: 02.9.5
fn f(n: 1..=5) -> Int:
    domain V = 0..n
    let c: 1..=5 = V.count
    c
print(f(4))
