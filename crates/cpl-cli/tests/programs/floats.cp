let n: Int = read()
var total = 0.0
var xs: List<F64> = []
for i in 0..n:
    let x: F64 = read()
    xs.push(x)
    total += x
print(total, total / n)
print(total, digits: 2)
print(sqrt(2.0), digits: 10)
print(xs)
print(floor(2.7), ceil(2.1), round(2.5), round(-2.5), floor(-2.7), abs(-1.5))
print(1.0 / 3.0, 2.0 ** 0.5, 1e3, 1.5e-3, 10.0 / 4.0)
print(3.0 < 4.0, max(1.5, 2.5), min(1.5, 2.5))
let half = 7 / 2
let fhalf = 7.0 / 2.0
print(half, fhalf, "{fhalf} and {total}")
print(0.1 + 0.2, digits: 17)
print(1e18, 123456789.125, digits: 3)
