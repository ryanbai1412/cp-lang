// expect: ok, output "3\n-1"
// A fn whose body ends in `while true` leaves only by a `return` inside the loop, so nothing
// follows the loop: every backend, the C++ included, builds it as a fn of its result type.
fn find(xs: List<Int>, x: Int) -> Int:
    var i = 0
    while true:
        if i == xs.len: return -1
        if xs[i] == x: return i
        i += 1
let xs = [4, 5, 6, 7]
print(find(xs, 7))
print(find(xs, 9))
