// expect: ok, output "1 3 5"
// rule: 06.3.2 06.1.7
// A closure in a generator that writes the generator's local by reference keeps the writes across each `yield` (adversarial round 47).
fn counts(n: Int) -> yields Int:
    var c = 0
    let bump = () => c += 1
    for i in 0..n:
        bump()
        yield c
        bump()
print([x for x in counts(3)])
