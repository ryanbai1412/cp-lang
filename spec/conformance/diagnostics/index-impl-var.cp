// expect: E0605
var n = 5
struct Id(i: Int)
impl Index for Id:
    let count = n // <- here
