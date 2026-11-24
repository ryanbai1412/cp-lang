// expect: E0505
// rule: 06.4.10
// An idiom lint is declared beside the fn it teaches: `@lint` on anything but a fn has no call to suggest (decision 512).
@lint(hand_rolled_pair, from: "(a, b)", to: "Pair(a, b)") // <- here
pub struct Pair(a: Int, b: Int)
print(Pair(1, 2).a)
