// expect: ok, output "7 0 1 0 3 0 1\n3 3\n1 2 3 0 1 2 3\n3\n1 5 7\n7 4\n1 2 1 4 1 2 1"
// rule: 04b.5.2 04b.5.3
// Every std/strings function that reads its input through `nth` takes a `[D]T` (cf1943b).
// Not `find_all`: its empty pattern yields the end key, which a `[D]T` has no value for
// (stdlib PLAN.md, General fixes owed).
use std.strings.{is_palindrome, lyndon_runs, overlap, lyndon_factorization, min_rotation, max_rotation, manacher}
domain P = 1..=7
let s: [P]Char = ['a', 'b', 'a', 'c', 'a', 'b', 'a']
let t: [P]Char = ['b', 'a', 'b', 'a', 'b', 'a', 'b']
assert s.starts_with("aba") and s.ends_with("ba") and s.starts_with("ca", at: 3)
assert is_palindrome(s) and not is_palindrome(t.drop(1))
assert lyndon_runs(t) == lyndon_runs("bababab")
print(z_function(s))
print(s.common_prefix_len("abax"), s.common_suffix_len("xaba"))
print(s.prefix_matches("aba"))
print(overlap(s, "abaxx"))
print(lyndon_factorization(s))
print(min_rotation(s)!, max_rotation(s)!)
print(manacher(s).odd)
