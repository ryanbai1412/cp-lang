// A fn passed where a one-argument fn is expected takes only its first
// parameter; the others need defaults (R06.1.8).
fn add(x: Int, y: Int) -> Int = x + y
fn pos(x: Int, strict: Bool) -> Bool = x > 0 or not strict
for v in [1, 2].map(add):
    print(v)
print([1, 2].filter(pos).count())
fn sum2(a: Int, b: Int) -> Int = a + b
for v in [(1, 2), (3, 4)].map(sum2):
    print(v)
