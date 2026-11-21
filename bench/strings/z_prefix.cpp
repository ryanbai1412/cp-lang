#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>

using i64 = int64_t;

int main() {
    const i64 n = 1000000;
    std::vector<i64> s(n, 0);
    i64 x = 123456789;
    for (i64 i = 0; i < n; i++) {
        x = (x * 1103515245 + 12345) & 0x7fffffff;
        s[i] = (x >> 16) % 3;
    }
    std::vector<i64> z(n, 0);
    z[0] = n;
    i64 l = 0, r = 0;
    for (i64 i = 1; i < n; i++) {
        i64 k = i < r ? std::min(r - i, z[i - l]) : 0;
        while (i + k < n && s[k] == s[i + k]) k++;
        z[i] = k;
        if (i + k > r) { l = i; r = i + k; }
    }
    std::vector<i64> p(n, 0);
    for (i64 i = 1; i < n; i++) {
        i64 k = p[i - 1];
        while (k > 0 && s[i] != s[k]) k = p[k - 1];
        if (s[i] == s[k]) k++;
        p[i] = k;
    }
    i64 ans = 0;
    for (i64 i = 0; i < n; i++) ans += z[i] * 3 + p[i];
    std::cout << ans << "\n";
}
