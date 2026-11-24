// expect: ok, output "YES"
print(yes(parallel_bsearch(1, 0..0, reset: () => (), apply: _ => (), ok: _ => true) == [None]))
