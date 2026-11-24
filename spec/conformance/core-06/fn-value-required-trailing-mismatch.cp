// expect: E0302
// rule: 06.1.8
fn add(x: Int, y: Int) -> Int = x + y
for v in [1, 2].map(add): // <- here
    print(v)
