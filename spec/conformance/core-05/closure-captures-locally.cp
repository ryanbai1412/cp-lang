// expect: ok, output "1"
// rule: 05.7.5
fn count() -> Int:
    var c = 0
    let f = () => c + 1
    f()
print(count())
