// expect: ok, output "7 5 6\n1 2 0\n2 0 3 1"
// `inverse` on a bijection writes each index where its image lands, from any index start.
let k: [5..8]0..3 = [1, 2, 0]
let m = k.inverse()
print(m[0], m[1], m[2])
enum Suit = Club | Heart | Spade
let g: [0..3]Suit = [Suit.Spade, Suit.Club, Suit.Heart]
let h = g.inverse()
print(h[Suit.Club], h[Suit.Heart], h[Suit.Spade])
let c: [0..4]1..=4 = [2, 4, 1, 3]
print(c.inverse())
