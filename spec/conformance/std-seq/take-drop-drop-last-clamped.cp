// expect: ok, output "5 4 2 6"
// rule: 07.4.11 07.7.3
fn nums() -> yields Int:
    for i in 1..=5: yield i

let a = [1, 2, 3, 4, 5]
print(a.take(10).len, a.drop_last(1).len, a.drop(3).len, nums().take(3).sum())
