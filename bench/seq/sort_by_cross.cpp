#include <bits/stdc++.h>
struct Turn { long long dx, dy, a, b; };
int main() {
    const long long n = 2500;
    long long s = 88172645;
    auto next = [&] { s = (s * 1103515245 + 12345) % 2147483648LL; return s % 1000000000; };
    std::vector<std::pair<long long, long long>> pts(n);
    for (auto& p : pts) { p.first = next(); p.second = next(); }
    std::stable_sort(pts.begin(), pts.end());
    std::vector<Turn> turns;
    turns.reserve(n * (n - 1) / 2);
    for (long long i = 0; i < n; i++)
        for (long long j = i + 1; j < n; j++)
            turns.push_back({pts[j].first - pts[i].first, pts[j].second - pts[i].second, i, j});
    std::stable_sort(turns.begin(), turns.end(), [](const Turn& u, const Turn& v) { return u.dx * v.dy - u.dy * v.dx > 0; });
    long long acc = 0;
    for (const Turn& t : turns) acc = (acc * 31 + t.a * n + t.b) % 1000000007;
    printf("%lld\n", acc);
}
