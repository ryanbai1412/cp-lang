// expect: ok, output "3"
// rule: 07.7.3
fn nat() -> yields Int:
    for i in 0..100: yield i
print(nat().take(3).sum())
