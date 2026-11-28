// expect: ok, output "same differ"
fn same<T: Hash>(a: T, b: T) -> Str = if a.hash() == b.hash(): "same" else: "differ"
print(same("ab", "ab"), same("ab", "ba"))
