struct P:
    c: Int
    fn pow(self, k: Int) -> Int = self.c * k

print(P(c: 5).pow(3), 2 ** 10)
