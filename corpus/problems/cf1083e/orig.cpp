// Codeforces 1083E (The Fair Nut and Rectangles): sort by x (so y falls), then
// dp[i] = x_i y_i - a_i + max(0, max_{j<i} dp[j] - x_j y_i) with a monotone convex hull.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
typedef __int128 L;
int main() {
    int n;
    scanf("%d", &n);
    vector<array<ll, 3>> r(n);
    for (auto& [x, y, a] : r) scanf("%lld %lld %lld", &x, &y, &a);
    sort(r.begin(), r.end());
    // lines f_j(q) = -x_j * q + dp[j]; slopes strictly fall, queries q = y_i fall.
    vector<pair<ll, ll>> hull = {{0, 0}};
    size_t head = 0;
    auto bad = [](pair<ll, ll> a, pair<ll, ll> b, pair<ll, ll> c) {
        // b is useless if intersection(a,c) makes b never max; slopes a.first > b.first > c.first
        return (L)(c.second - a.second) * (a.first - b.first) >= (L)(b.second - a.second) * (a.first - c.first);
    };
    ll best = 0;
    for (auto [x, y, a] : r) {
        while (head + 1 < hull.size() &&
               (L)hull[head + 1].first * y + hull[head + 1].second >= (L)hull[head].first * y + hull[head].second)
            head++;
        ll dp = x * y - a + (ll)((L)hull[head].first * y + hull[head].second);
        best = max(best, dp);
        pair<ll, ll> ln = {-x, dp};
        while (hull.size() - head >= 2 && bad(hull[hull.size() - 2], hull.back(), ln)) hull.pop_back();
        hull.push_back(ln);
        if (head >= hull.size()) head = hull.size() - 1;
    }
    printf("%lld\n", best);
}
