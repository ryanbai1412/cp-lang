// expect: ok, output "5 6 0"
// rule: 04b.5.2
// An all-`?` pattern matches at each domain key it fits from; one longer than the text matches nowhere.
domain D = 5..=7
let s: [D]Char = ['x', 'y', 'z']
print(s.find_all_wildcard("??").collect(), s.find_all_wildcard("????").collect().len)
