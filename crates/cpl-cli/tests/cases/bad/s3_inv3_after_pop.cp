// Anti-spookiness Invariant 3: calling a mutating method (`q.pop()`) invalidates Rule N2 so a second `q.pop()` stays `?Int`.
var q: List<Int> = [1, 2]
while not q.is_empty:
    let a: Int = q.pop()
    let b: Int = q.pop()
    print(a + b)
