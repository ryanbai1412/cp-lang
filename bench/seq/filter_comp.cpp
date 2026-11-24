#include <bits/stdc++.h>
struct R { long long cost, v, p; };
int main() {
    const long long n = 20000000;
    std::vector<R> ks;
    ks.reserve(n);
    for (long long i = 0; i < n; i++)
        if (((i * 7919993) & 15) < 5) ks.push_back({i * 3, i, i / 2});
    printf("%lld\n", (long long)ks.size() + ks[0].cost + ks.back().p);
}
