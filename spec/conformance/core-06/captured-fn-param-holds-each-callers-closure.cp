// expect: ok, output "7 45"
// rule: 06.1.7
// A closure capturing a fn parameter calls whichever closure each caller passed, not the first one devirtualization saw (adversarial round 47).
fn twice_of(f: (Int) -> Int, x: Int) -> Int:
    let g = (y: Int) => f(f(y))
    g(x)
print(twice_of(x => x + 1, 5), twice_of(x => x * 3, 5))
