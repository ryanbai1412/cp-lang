// `Int` and `I128` `+`, `-` and `*` the compiler can't prove in range,
// close to the edges: exact in every build, never C++ overflow (R09.3.4).
// In `unchecked:` and `checks(overflow: off)`, `x op= y` wraps as
// `x = x op y` does (R09.4.2, R09.4.3).
let xs = [9223372036854775806, -9223372036854775806, 3037000499]
print(xs[0] + 1, xs[1] - 1, xs[2] * xs[2])
var acc = xs[0]
acc += 1
acc -= xs[0]
acc *= -4611686018427387903
print(acc)
let w: I128 = I128(xs[0]) * I128(xs[0])
print(w + w, w - I128(xs[2]))
print(xs.map(x => x * 1 - 1))
var y = xs[0]
unchecked:
    y += 9
    y *= 3
with checks(overflow: off):
    y -= xs[1]
    y = y * 5
print(y)
