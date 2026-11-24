# Friction report: dmopc21c6p2

No gap. The runs of 0s are `s.split('1')`, sorted descending and summed, with no sentinel character appended. The old port was wrong: on `26 3 / 11101010100110110100011001` it printed 3, while the accepted C++ and this port print 7 (3 + 2 + 2). This port agrees with the accepted C++ on 30 generated cases and at max.

## Measurements

gen.py --max (n = 1e6): port 0.02 s, accepted C++ 0.01 s.
