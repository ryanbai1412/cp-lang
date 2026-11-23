let n: 1..=100 = read()
var xs: List<Int> = []
for i in 0..n:  // expect: push_loop
    xs.push(i * i)
