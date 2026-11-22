// A table of `rand` values takes its element type from the declaration.
let stations: 1..=50 = read()
domain E = 0..4
var at: [E]1..=stations = [E] e => rand(1, stations)
print(at.len, sorted([E] e => rand(1, stations)).len)
