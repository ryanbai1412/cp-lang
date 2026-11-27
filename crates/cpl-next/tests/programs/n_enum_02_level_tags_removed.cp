// expect: E0503
// An enum's tokens are spelled one way, `@tag` on each variant; the enum-level `@tags` is
// removed (R02.7.2, decision 459).
@tags("U", "D") // <- here
enum Dir = Up | Down
print(Dir.Up)
