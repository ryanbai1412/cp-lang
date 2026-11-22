// expect: ok, output "11\n12"
// rule: 06.1.8
fn bump(x: Int, by: Int = 10) -> Int = x + by
for y in [1, 2].map(bump):
    print(y)
