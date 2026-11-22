// expect: ok, output "4999"
// rule: 03.1.1
fn f(n: 1..=5000) -> Int:
    domain V = 0..n
    let parent = [V] v => v
    Int(parent[V(n - 1)])
print(f(5000))
