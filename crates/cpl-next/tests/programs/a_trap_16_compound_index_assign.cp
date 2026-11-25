// expect: R0202
// rule: contract 4: place once, then value, one read, one write
// Compound assign to an indexed element: index evaluated once, overflow at the line.
var xs = [9223372036854775807, 0]
var i = 0
xs[i] += 1  // <- here
print(xs[0])
