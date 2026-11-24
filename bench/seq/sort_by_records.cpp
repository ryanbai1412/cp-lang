#include <bits/stdc++.h>
struct R { long long cost, v, p; };
int main() {
    const long long n = 1000000;
    long long acc = 0;
    for (long long r = 0; r < 4; r++) {
        std::vector<R> xs(n);
        for (long long i = 0; i < n; i++) xs[i] = {(i * 7919993 + r) % 1000000007, i, i / 2};
        std::stable_sort(xs.begin(), xs.end(), [](const R& a, const R& b) { return a.cost > b.cost; });
        acc += xs[0].v + xs[n - 1].v + xs[n / 2].p;
    }
    printf("%lld\n", acc);
}
