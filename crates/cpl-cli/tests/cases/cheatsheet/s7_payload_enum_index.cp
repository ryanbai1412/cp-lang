enum Node:
    Source
    Build(s: 0..3)
var f: [Node]Int = 0
f[Node.Build(2)] = 1
print(Node.count)
