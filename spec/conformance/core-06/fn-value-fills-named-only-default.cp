// expect: ok, output "2\n4"
// rule: 06.1.8
fn scale(x: Int; by: Int = 2) -> Int = x * by
for y in [1, 2].map(scale):
    print(y)
