// expect: ok, output "7"
// rule: 01.3
// `return` inside `repeat` leaves the fn, as inside `for _ in 0..k`.
fn f() -> Int:
    repeat 3:
        return 7
    0
print(f())
