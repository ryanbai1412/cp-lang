// IOI 2010 Hotter Colder. No accepted submission exists. Jill's number lies in [lo, hi];
// after guessing p, guessing g learns which side of (p + g) / 2 it is on (or that it is the
// midpoint). So each guess reflects p about the interval's centre, g = lo + hi - p, which
// halves the interval, clamped to [1, N] near the ends. The main stands in for the grader:
// Q, then per game N; the program prints `guess G` (each answered 1, -1 or 0) and `answer X`.
#include <bits/stdc++.h>
using namespace std;

int Guess(int G);

int HC(int n) {
    long long lo = 1, hi = n, p = 1;
    if (n == 1) return 1;
    Guess(1);
    while (lo < hi) {
        long long g = min<long long>(n, max<long long>(1, lo + hi - p));
        if (g == p) g = p > 1 ? p - 1 : p + 1;
        int r = Guess(g);
        long long s = p + g;
        if (r == 0 && s % 2 == 0) return s / 2;
        bool high = (g > p) == (r > 0);
        if (r == 0) high = false;  // no integer midpoint: same cannot happen
        if (high) lo = max(lo, s / 2 + 1);
        else hi = min(hi, (s - 1) / 2);
        p = g;
    }
    return lo;
}

// stdio grader
int Guess(int G) {
    printf("guess %d\n", G);
    fflush(stdout);
    int r;
    if (scanf("%d", &r) != 1) exit(1);
    return r;
}

int main() {
    int q;
    if (scanf("%d", &q) != 1) return 1;
    while (q--) {
        int n;
        if (scanf("%d", &n) != 1) return 1;
        printf("answer %d\n", HC(n));
        fflush(stdout);
    }
}
