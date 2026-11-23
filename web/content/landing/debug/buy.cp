// Best profit from one buy and one later sell.
let n: 1..=2e5 = read()
let p: [0..n]1..=1e9 = read()
var low = p[0]
var best = 0
for x in p:
    best max= x - low
    low max= x
print(best)

@gen
fn make_case(size: Int):
    let n = rand(1, size)
    print(n)
    print([rand(1, 20) for _ in 0..n])

@alt(program)
fn slow():
    var best = 0
    for i in 0..n:
        for j in i..n:
            best max= p[j] - p[i]
    print(best)
