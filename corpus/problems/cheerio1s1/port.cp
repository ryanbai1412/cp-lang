// Cheerio Contest 1 S1, traffic lights on a road (buildable today-form)
// original: corpus/submissions/5337471.cpp
let n: 1..=2e5 = read()
let k: 1..=1e9 = read()
type Light = (p: 1..=1e9, r: 1..=1e9, g: 1..=1e9)
let lights: [0..n]Light = read()

var pos = 0
var t = 0
for (p, r, g) in lights:
    t += p - pos
    pos = p
    let phase = t % (r + g)
    if phase < r: t += r - phase
print(t + k - pos)
