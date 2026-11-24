#include <bits/stdc++.h>
struct Edge { long long u, v, w; };
int main() {
    const int m = 250000;
    std::vector<Edge> es(m);
    for (long long e = 0; e < m; e++) es[e] = {e % 500, e / 500, e * 7919 % 1000003};
    std::vector<int> order(m);
    std::iota(order.begin(), order.end(), 0);
    std::stable_sort(order.begin(), order.end(), [&](int a, int b) { return es[a].w < es[b].w; });
    printf("%lld %lld\n", es[order[0]].w, es[order[m - 1]].w);
}
