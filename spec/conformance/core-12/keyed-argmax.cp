// expect: ok, output "1 8"
// gap: compiler 2026-11-24T05Z
// rule: 12.4.2
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i

print([3, 9, 2].argmax()!, evens(10).keyed(by: v => v).argmax()!)
