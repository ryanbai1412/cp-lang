struct A:
    x: Int

struct B:
    y: Int

    fn size(self) -> Int = self.y * 10

    fn find(self, k: Int) -> Int = self.y + k

enum Shape:
    Dot
    Box(Int)

fn size(self: A) -> Int = self.x + 1

fn find(self: A, k: Int) -> Int = self.x - k

fn upper(self: Shape) -> Int:
    match self:
        Dot => 0
        Box(w) => w

fn sign(self: B) -> Int = 7

let a = A(4)
let b = B(5)
print(a.size(), b.size(), a.find(1), b.find(1))
print("hello".find("l"), [3, 1, 2].find(1))
print(Shape.Box(9).upper(), 'a'.upper(), b.sign(), (-3).sign(), 3.sign())
