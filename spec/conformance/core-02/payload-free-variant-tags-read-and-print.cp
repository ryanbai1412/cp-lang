// expect: ok, output "D U"
// rule: 02.7.2
// input: "D U\n"
// A payload-free enum names its tokens with `@tag` on each variant, as one with payloads does.
enum Dir = @tag("U") Up | @tag("D") Down
let a = read<Dir>()
let b = read<Dir>()
print(a, b)
