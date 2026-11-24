// A `yields` fn that feeds itself, or two that feed each other, runs on
// the VM like any fed fn, and an unwrap in the fed body, whose value a
// `do` gives, does not leave the loop.
fn walk(n: Int) -> yields Int:
    if n > 0:
        yield n
        for x in walk(n - 1): yield x

fn evens(n: Int) -> yields Int:
    if n >= 0:
        if n % 2 == 0: yield n
        for x in odds(n - 1): yield x

fn odds(n: Int) -> yields Int:
    if n >= 0:
        for x in evens(n - 1): yield x

let n: 0..=50 = read()
var s = 0
for x in walk(n):
    if x == 3: break
    s += x
var t = 0
for x in evens(n): t += x
print(s, t)

fn cuts(n: Int) -> yields (lost: ?Int, gained: ?Int):
    for i in 0..n: yield (lost: Some(i * i), gained: if i % 2 == 0: None else: Some(i))
for c in cuts(3): print(c.lost!, yes(c.gained == None))
