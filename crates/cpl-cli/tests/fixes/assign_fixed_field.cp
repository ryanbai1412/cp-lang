struct Stack:
    items: List<Int>
    top: Int
var s = Stack(items: [], top: 0)
s.top = 1
print(s.top)
