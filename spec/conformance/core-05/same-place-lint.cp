// expect: L0601
// rule: 05.8.4
var xs = [1, 2]
xs[0] = xs.pop()! // <- here
