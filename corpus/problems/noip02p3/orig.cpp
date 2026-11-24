// Not a judge-accepted submission: written for the corpus and checked against the samples and an exhaustive brute.
#include <bits/stdc++.h>
using namespace std;
using ld = long double;
int main() {
    const ld eps = 1e-4;
    ld h, s1, v, l, k;
    int n;
    cin >> h >> s1 >> v >> l >> k >> n;
    // the ball's height when the car's front is at x (the car starts at s1 and drives toward 0)
    auto height = [&](ld x) { ld t = (s1 - min(x, s1)) / v; return h - 5 * t * t; };
    int caught = 0;
    for (int i = 0; i < n; i++) {
        if (i - l - eps > s1) continue;  // behind the car from the start: never under it
        // the ball is over the car from when its front reaches i until its back passes i
        ld lo = height(i - l - eps), hi = height(i + eps);
        if (lo <= k + eps && hi >= 0) caught++;
    }
    printf("%d\n", caught);
}
