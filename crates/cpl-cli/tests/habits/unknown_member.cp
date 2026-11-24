struct Id(i: Int)
impl Index for Id:
    let count = 3
    @get fn ord(self) -> Int = i
    fn nth(k: Int) -> Id = Id(k)
print(Id.K)  // <- here
