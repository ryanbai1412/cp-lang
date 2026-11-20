enum Node:
    Source
    Sink
    Build(s: Int)
let nd = Node.Build(3)
print(nd == Node.Sink)
