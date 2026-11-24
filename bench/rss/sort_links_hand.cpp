#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<int> f(n), g(n);
    for (int& x : f) if (scanf("%d", &x) != 1) return 1;
    for (int& x : g) if (scanf("%d", &x) != 1) return 1;
    std::vector<std::array<int, 3>> links;
    links.reserve(2 * (size_t)n);
    for (int i = 0; i < n; i++) links.push_back({f[i], 0, i});
    for (int i = 0; i < n; i++) links.push_back({g[i], 1, i});
    std::stable_sort(links.begin(), links.end(), [](const auto& x, const auto& y) { return x[0] < y[0]; });
    long long tot = 0;
    for (long long i = 0; i < (long long)links.size(); i++)
        tot += links[i][1] == 0 ? i * links[i][2] : -i * links[i][2];
    printf("%lld\n", tot);
}
