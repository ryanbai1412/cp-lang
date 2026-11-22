// error: expected Int, got ?Int
// Anti-spookiness Invariant 3: narrowing is NOT born at control-flow joins.
var x: ?Int = none
let flag: 0..=1 = read()
if flag == 1:
    if x == none:
        x = 7
let y: Int = x
print(y)
