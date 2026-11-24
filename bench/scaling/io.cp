// n: 131072
// stdin: 3
let n: Int = read()
let reps = 1 << 18
var s = 0
for i in 0..reps:
    s += yes(i % 3 == 0).len // timed: io.yes
for _ in 0..2:
    s += read_all<Int>(until: x => x < 0).len // timed: io.read_all [let: tokens_read = n]
for _ in 0..reps:
    let w = Wire() // timed: io.Wire.new
let wire = Wire()
for _ in 0..n:
    let x: Int = wire.ask("?") // timed: io.Wire.ask [fixed: query]
    s += x
print(s)
