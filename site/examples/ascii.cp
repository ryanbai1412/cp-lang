let n: 1..=100 = read()
domain S = 1..=n
let word: [S]Char = read()
print(count(word[i] == 'a' for i in S))
