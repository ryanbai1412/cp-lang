// expect: E0503 at 2
// rule: 02.7.2
// There is one spelling for an enum's tokens: `@tag` on each variant (decision 459).
@tags("U", "D")
enum Dir = Up | Down
print(Dir.Up)
