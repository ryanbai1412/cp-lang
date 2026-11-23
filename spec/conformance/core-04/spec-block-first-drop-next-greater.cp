// expect: ok, output "1 2"
// rule: 04b.1.11
fn first_drop<K>(a: [K]Int) -> ?K =
    a.keys.drop(1).find(i => a[i] < a[i.prev!])

fn next_greater<K>(a: [K]Int) -> [K]?K:  // keyed like a
    var res: [K]?K = [a.keys].filled(None)
    var stack = List<K>()
    for i in a.keys:
        while let j = stack.last and a[j] < a[i]:
            res[j] = i
            stack.pop()
        stack.push(i)
    res
let a = [3, 1, 4, 1]
print(first_drop(a)!, next_greater(a)[0]!)
