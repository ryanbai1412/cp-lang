// expect: ok, output "2 2 1\n3"
fn sizes<S: Indexed>(s: S, k: Int) -> List<Int> = [c.len for c in s.chunks(k)]
print(sizes([1, 2, 3, 4, 5], 2))
print(sizes("abcab", 2).len)
