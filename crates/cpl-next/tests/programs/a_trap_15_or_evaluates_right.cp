// expect: R0203
// rule: R01.5 `and`/`or` lazy right side
// `and` short-circuits: the right side is not evaluated; `or` evaluates it here.
let z = 0
let ok = z != 0 and 1 / z > 0
let bad = z != 0 or 1 / z > 0  // <- here
print(if ok: 1 else: 0, if bad: 1 else: 0)
