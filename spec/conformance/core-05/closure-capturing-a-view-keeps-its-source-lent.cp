// expect: E0602
// rule: 05.7.1
// A closure that captures a view holds the view, so its source stays lent while the closure lives (adversarial round 46).
var xs = [1, 3]
let v = xs.filter(y => y > 0)
let f = () => [x for x in v].len
xs.push(1) // <- here
print(f())
