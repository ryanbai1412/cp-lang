// expect: E0302 at 2
// rule: 06.1.8
fn add(x: Int, y: Int) -> Int = x + y
for v in [1, 2].map(add):
    print(v)
