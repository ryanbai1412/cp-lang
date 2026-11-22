// Synthesis #2 rule 2: `read()` and `ask()` take their type from where the value goes.
let n: 1..=10 = read()
domain Day = 1..=n
var a: [Day]Int = 0
var b: [Day]0..=9 = 0
var rate: [Day]F64 = 0.0
for d in Day: (a[d], b[d], rate[d]) = read()      // typed places, left to right

fn next_small() -> 0..=100 = read()                // the return type
fn twice(x: Int, y: 0..=5) -> Int = x * 2 + y

var k = 0
k = read::<Int>() + 1                              // no context: the one explicit spelling
let q = next_small()
let t = twice(read(), read())                      // parameters
var cur: Int = 0
cur = read()                                       // a typed place
var who = Day(1)
who = read()                                       // a domain value is read by its label
print(sum(a), sum(b), "{sum(rate):.2f}", k, q, t, cur, who)

fn guess(x: Int) -> 0..=2 = ask("?", x)            // ask: print, flush, read the reply as the return type
let r1 = guess(5)
var r2 = 0
r2 = ask("?", 6)
print(r1, r2)
