// expect: ok, output "AB|ab|a b|a b |  a b"
// rule: 12.13.2
// `@aka` is an ordinary decorator any library writes; the std fns it marks run as written.
@aka(shout, yell)
fn loud(self: Str) -> Str = self.to_upper()

let s = "  a b "
print("{"ab".loud()}|{"AB".to_lower()}|{s.trim()}|{s.trim_start()}|{s.trim_end()}")
