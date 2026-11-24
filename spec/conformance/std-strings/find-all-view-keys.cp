// expect: ok, output "3 5\n3\n5\n5"
// rule: 04b.5.2 04b.5.1
let w = "banana"[2..]
print(find_all(w, "a").collect())
print(w.search('a')!)
print(w.rsearch('a')!)
print(w.rsearch("")!)
