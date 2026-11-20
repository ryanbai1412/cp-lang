// cp: 0.1 c217120+dirty
// IOI 2004 Phidias: cut a w x h slab into plates of the wanted sizes with
// straight edge-to-edge cuts. Print the least area that goes to waste.
let w: 1..=600 = read()
let h: 1..=600 = read()
let n: 1..=200 = read()
let plates: [n](w: 1..=w, h: 1..=h) = read()
let wanted = Set(plates)

// A piece is a plate, or scrap, or it is cut in two. By symmetry the first
// part of a cut is never the larger one.
@memo
fn waste(a: 1..=w, b: 1..=h) -> Int:
    if (a, b) in wanted: return 0
    var best = a * b
    for x in 1..=a / 2: best min= waste(x, b) + waste(a - x, b)
    for y in 1..=b / 2: best min= waste(a, y) + waste(a, b - y)
    best
print(waste(w, h))
