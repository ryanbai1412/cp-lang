#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>

int main() {
    const int n = 200000;
    std::vector<int64_t> v(n);
    int64_t seed = 987654321;
    for (int i = 0; i < n; i++) {
        seed = (seed * 1103515245LL + 12345LL) % 2147483647LL;
        v[i] = seed % 1000003;
    }

    int64_t acc = 0;
    for (int r = 0; r < 200; r++) {
        int k = (r * 131071 + 37) % n;
        std::rotate(v.begin(), v.begin() + k, v.end());
        acc += v[0] + v[n / 2] + v[n - 1];
        int rk = k / 3;
        std::rotate(v.begin(), v.begin() + (n - rk), v.end());
        acc += v[1] + v[n - 2];
    }

    std::cout << acc << '\n';
    return 0;
}
