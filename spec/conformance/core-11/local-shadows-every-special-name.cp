// expect: ok, output "300 301 302 303 304 305 306 307 308 309 310 311 312 313 314 315 316 317 318 319 320 321"
// rule: 11.2.3 11.3.1
// Every name the compiler special-cases (`debt-ratchet.py --list prelude_privilege`) yields to a local fn of that name.
fn abs(x: Int) -> Int = x * 100 + 0
fn adjacency(x: Int) -> Int = x * 100 + 1
fn all(x: Int) -> Int = x * 100 + 2
fn any(x: Int) -> Int = x * 100 + 3
fn argmax(x: Int) -> Int = x * 100 + 4
fn argmin(x: Int) -> Int = x * 100 + 5
fn count(x: Int) -> Int = x * 100 + 6
fn counts(x: Int) -> Int = x * 100 + 7
fn find(x: Int) -> Int = x * 100 + 8
fn first(x: Int) -> Int = x * 100 + 9
fn last(x: Int) -> Int = x * 100 + 10
fn len(x: Int) -> Int = x * 100 + 11
fn max(x: Int) -> Int = x * 100 + 12
fn min(x: Int) -> Int = x * 100 + 13
fn prefix(x: Int) -> Int = x * 100 + 14
fn shuffle(x: Int) -> Int = x * 100 + 15
fn sort(x: Int) -> Int = x * 100 + 16
fn sorted(x: Int) -> Int = x * 100 + 17
fn span(x: Int) -> Int = x * 100 + 18
fn sqrt(x: Int) -> Int = x * 100 + 19
fn sum(x: Int) -> Int = x * 100 + 20
fn take(x: Int) -> Int = x * 100 + 21
print(abs(3), adjacency(3), all(3), any(3), argmax(3), argmin(3), count(3), counts(3), find(3), first(3), last(3), len(3), max(3), min(3), prefix(3), shuffle(3), sort(3), sorted(3), span(3), sqrt(3), sum(3), take(3))
