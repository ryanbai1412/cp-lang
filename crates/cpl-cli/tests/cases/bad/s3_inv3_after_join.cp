// error: `y` is declared an Int, but `x` is an ?Int
// Anti-spookiness Invariant 3: narrowing is NOT born at control-flow joins.
var x: ?Int = None
let flag: 0..=1 = read()
if flag == 1:
    if x == None:
        x = 7
let y: Int = x
print(y)
