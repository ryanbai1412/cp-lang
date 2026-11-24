let n: 1..=100_000 = read()
let reps: 1..=1000 = read()
let a: [0..n]0..10_000 = read()
var acc = 0
for r in 0..reps:
    var st: List<Int> = []
    for x in a:
        if let t = st.last:
            if t > x: acc += 1
        if x % 3 != r % 3: st.push(x)
    acc += st.last ?? 0
    acc += st.first!
print(acc)
