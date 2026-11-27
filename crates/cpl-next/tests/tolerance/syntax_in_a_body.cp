// A fn whose body does not parse: every other unit still types and lowers, and a call
// of it is checked against its signature.
fn good(a: Int) -> Int:
    a + 1

fn broken(a: Int) -> Int: // injected
    let x = (a +
    x

fn calls_broken() -> Int:
    broken(2) + good(3)

print(good(1))
print(calls_broken())
