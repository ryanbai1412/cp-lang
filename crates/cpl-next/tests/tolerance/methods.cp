// Errors in one method of a struct leave its other methods and their callers typed.
struct Counter:
    n: Int
    fn get(self) -> Int:
        self.n
    fn bad(self) -> Bool: // injected
        self.n + 1
    fn worse(self) -> Int: // injected
        self.missing
let c = Counter(4)
print(c.get())
