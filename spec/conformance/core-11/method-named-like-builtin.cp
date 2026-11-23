// expect: ok, output "300 301 302 303 304 305 306 307 308 309 310 311 312 313 314 315 316 317 318 319 320 321"
// rule: 11.2.3
// A user method named like a compiler-special name runs its own body, not the builtin.
struct W:
    v: Int
    pub fn abs(self) -> Int = self.v * 100 + 0
    pub fn adjacency(self) -> Int = self.v * 100 + 1
    pub fn all(self) -> Int = self.v * 100 + 2
    pub fn any(self) -> Int = self.v * 100 + 3
    pub fn argmax(self) -> Int = self.v * 100 + 4
    pub fn argmin(self) -> Int = self.v * 100 + 5
    pub fn count(self) -> Int = self.v * 100 + 6
    pub fn counts(self) -> Int = self.v * 100 + 7
    pub fn find(self) -> Int = self.v * 100 + 8
    pub fn first(self) -> Int = self.v * 100 + 9
    pub fn last(self) -> Int = self.v * 100 + 10
    pub fn len(self) -> Int = self.v * 100 + 11
    pub fn max(self) -> Int = self.v * 100 + 12
    pub fn min(self) -> Int = self.v * 100 + 13
    pub fn prefix(self) -> Int = self.v * 100 + 14
    pub fn shuffle(self) -> Int = self.v * 100 + 15
    pub fn sort(self) -> Int = self.v * 100 + 16
    pub fn sorted(self) -> Int = self.v * 100 + 17
    pub fn span(self) -> Int = self.v * 100 + 18
    pub fn sqrt(self) -> Int = self.v * 100 + 19
    pub fn sum(self) -> Int = self.v * 100 + 20
    pub fn take(self) -> Int = self.v * 100 + 21
let w = W(3)
print(w.abs(), w.adjacency(), w.all(), w.any(), w.argmax(), w.argmin(), w.count(), w.counts(), w.find(), w.first(), w.last(), w.len(), w.max(), w.min(), w.prefix(), w.shuffle(), w.sort(), w.sorted(), w.span(), w.sqrt(), w.sum(), w.take())
