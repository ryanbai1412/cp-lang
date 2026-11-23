// expect: E0605 at 4
var n = 5
struct Id(i: Int)
impl Index for Id:
    let count = n
