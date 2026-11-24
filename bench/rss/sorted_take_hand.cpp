#include <algorithm>
#include <cstdio>
#include <vector>
using ll = long long;
int main() {
    ll n;
    if (scanf("%lld", &n) != 1) return 1;
    std::vector<std::pair<ll, ll>> ups(n);
    for (ll i = 0; i < n; i++) ups[i] = {i * 7919 % 1000003, i};
    std::stable_sort(ups.begin(), ups.end(), [](auto& a, auto& b) { return a.first > b.first; });
    ups.resize(n / 2);
    std::vector<ll> cells(4 * n);
    for (ll i = 0; i < 4 * n; i++) cells[i] = ups[i % (n / 2)].first + i;
    printf("%lld\n", cells[4 * n - 1] + cells[0]);
}
