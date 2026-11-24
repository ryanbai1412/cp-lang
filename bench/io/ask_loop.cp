let wire = Wire()
var s = 0
for i in 0..100_000:
    let x: Int = wire.ask("? {i}")
    s += x
print("! {s}")
