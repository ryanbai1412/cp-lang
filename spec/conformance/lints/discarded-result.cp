// expect: L0502
fn work(u: Int) -> Int:
    print(u)
    u
let _ = work(2) // <- here
