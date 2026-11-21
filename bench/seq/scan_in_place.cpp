#include <algorithm>
#include <cstdint>
#include <iostream>
#include <numeric>
#include <vector>

int main() {
    const int n = 200000;
    std::vector<int64_t> v(n);
    int64_t seed = 123456789;
    for (int i = 0; i < n; i++) {
        seed = (seed * 1103515245LL + 12345LL) % 2147483647LL;
        v[i] = (seed % 2001) - 1000;
    }

    int64_t acc = 0;
    for (int r = 0; r < 400; r++) {
        std::partial_sum(v.begin(), v.end(), v.begin(),
                         [](int64_t x, int64_t y) { return std::min(x, y); });
        acc += v[n - 1 - r];
        std::partial_sum(v.rbegin(), v.rend(), v.rbegin(),
                         [](int64_t x, int64_t y) { return std::max(x, y); });
        acc += v[r];
        v[r] = r - 1000;
    }

    std::cout << acc << '\n';
    return 0;
}
