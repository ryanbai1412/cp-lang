// expect: ok, output "3"
// rule: std.hashmap.Counter
struct Counter:
    n: Int
print(Counter(n: 3).n)
