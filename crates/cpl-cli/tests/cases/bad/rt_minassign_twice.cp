// error: would evaluate `a[nx(...)]` twice
var c = 0
fn nx() -> Int:
    c += 1
    return c
var a: [0..9]Int = 0
a[nx()] max= 4
print(a)
