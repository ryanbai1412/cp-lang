domain I = 0..5
var scale = 1
@memo
fn f(i: I) -> Int:
    i.ord * scale
scale = 2
print(f(I.first))
