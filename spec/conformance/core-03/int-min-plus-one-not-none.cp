// expect: ok, output "-9223372036854775807 1"
// rule: 03.2.1
let xs: List<?Int> = [Some(-9223372036854775807), None]
print(xs[0]!, if xs[1] is None: 1 else: 0)
