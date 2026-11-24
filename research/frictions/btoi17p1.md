# Friction report: btoi17p1

Rethought. The v0.2 port tried every subset of the remaining neighbours as a hand bit mask, checking each pair against a Set, which is O(2^K K^2) lookups per person. Its header carried an "as it builds today" note. The v0.3 port names the remaining neighbours' domain `Near` and keeps their friendships as `[Near]Subset<Near>`. It fills a `[Near.subsets]Bool` clique table: a subset is a clique when its first member knows the rest and the rest is a clique. gen.py is new.

No language gap.

## Measurements

gen.py --max (n = 5e4, K = 10, 0..9 earlier links each): port 0.16 s, accepted C++ 0.29 s, same output. With all 9 links per person the accepted C++ takes 8.9 s, so --max does not use that shape.
