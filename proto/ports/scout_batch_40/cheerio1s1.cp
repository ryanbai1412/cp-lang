// Cheerio Contest 1 S1, traffic lights on a road
// original: corpus/submissions/5337471.cpp
// Drive from 0 to k at speed 1. Light i sits at p, is red for the first r
// seconds of every r + g cycle, and holds the car until it turns green.
let (n, k): (1..=2e5, 1..=1e9) = read()
type Light = (p: 1..=k, r: 1..=1e9, g: 1..=1e9)
let lights: [0..n]Light = read()

var (pos, t) = (0, 0)
for (p, r, g) in lights:
    t += p - pos
    pos = p
    let phase = t % (r + g)
    if phase < r: t += r - phase
print(t + k - pos)
