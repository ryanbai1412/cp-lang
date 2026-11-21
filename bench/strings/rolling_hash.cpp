#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>

using i64 = int64_t;
static constexpr i64 M1 = 1000000007;
static constexpr i64 M2 = 1000000009;

struct RollingHash {
    std::vector<i64> p1, p2, h1, h2;
    RollingHash(const std::vector<i64>& s, i64 b1, i64 b2) {
        i64 n = (i64)s.size();
        p1.assign(n + 1, 1);
        p2.assign(n + 1, 1);
        h1.assign(n + 1, 0);
        h2.assign(n + 1, 0);
        for (i64 i = 0; i < n; i++) {
            p1[i + 1] = (p1[i] * b1) % M1;
            p2[i + 1] = (p2[i] * b2) % M2;
            i64 v = s[i] + 1;
            h1[i + 1] = (h1[i] * b1 + v) % M1;
            h2[i + 1] = (h2[i] * b2 + v) % M2;
        }
    }
    std::pair<i64, i64> slice(i64 l, i64 r) const {
        i64 a = (h1[r] - h1[l] * p1[r - l]) % M1;
        if (a < 0) a += M1;
        i64 b = (h2[r] - h2[l] * p2[r - l]) % M2;
        if (b < 0) b += M2;
        return {a, b};
    }
    bool eq_sub(i64 a, i64 b, i64 len) const {
        i64 a1 = (h1[a + len] - h1[a] * p1[len]) % M1;
        if (a1 < 0) a1 += M1;
        i64 b1 = (h1[b + len] - h1[b] * p1[len]) % M1;
        if (b1 < 0) b1 += M1;
        if (a1 != b1) return false;
        i64 a2 = (h2[a + len] - h2[a] * p2[len]) % M2;
        if (a2 < 0) a2 += M2;
        i64 b2 = (h2[b + len] - h2[b] * p2[len]) % M2;
        if (b2 < 0) b2 += M2;
        return a2 == b2;
    }
    i64 lcp(i64 a, i64 b) const {
        i64 n = (i64)p1.size() - 1;
        i64 lo = 0, hi = n - std::max(a, b);
        while (lo < hi) {
            i64 mid = (lo + hi + 1) >> 1;
            if (eq_sub(a, b, mid)) lo = mid;
            else hi = mid - 1;
        }
        return lo;
    }
};

int main() {
    const i64 n = 500000;
    std::vector<i64> s(n, 0);
    i64 x = 987654321;
    for (i64 i = 0; i < n; i++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        s[i] = (x >> 16) % 4;
    }
    RollingHash rh(s, 911382323, 972663749);
    i64 ans = 0;
    for (i64 q = 0; q < 200000; q++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        i64 a = (x >> 8) % n;
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        i64 b = (x >> 8) % n;
        i64 l = std::min(a, b), r = std::max(a, b);
        auto [v1, v2] = rh.slice(l, r);
        ans ^= v1 + v2 + rh.lcp(a, b);
    }
    std::cout << ans << "\n";
}
