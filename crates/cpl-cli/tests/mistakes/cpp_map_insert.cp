// here: 4
// C++'s `m.insert({k, v})` on a Map (self-run mistakes): a set-brace error, then a type error.
var m: Map<Int, Int> = Map()
m.insert({1, 2})
print(m[1])
