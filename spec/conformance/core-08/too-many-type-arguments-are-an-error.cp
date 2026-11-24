// expect: E0303
// rule: 08.2.5
struct Q<T>:
    x: T
let q: Q<Int, Int> = Q(1) // <- here
print(q.x)
