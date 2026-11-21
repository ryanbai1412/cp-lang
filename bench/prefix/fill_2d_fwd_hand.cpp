#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

static constexpr uint32_t MOD = 998244353;

static inline uint32_t addm(uint32_t a, uint32_t b) {
    uint32_t s = a + b;
    return s >= MOD ? s - MOD : s;
}
static inline uint32_t subm(uint32_t a, uint32_t b) {
    return a >= b ? a - b : a + MOD - b;
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int64_t h, w, seed;
    if (!(cin >> h >> w >> seed)) return 0;
    int64_t W = w + 1;
    vector<uint32_t> p((h + 1) * W, 0);
    int64_t s = seed;
    for (int64_t r = 0; r < h; ++r) {
        for (int64_t c = 0; c < w; ++c) {
            s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
            uint32_t val = 1;
            if (r > 0 && c > 0) {
                int64_t r0 = s & (r - 1);
                s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
                int64_t c0 = s & (c - 1);
                val = addm(addm(subm(subm(p[r * W + c], p[r0 * W + c]), p[r * W + c0]), p[r0 * W + c0]), 1);
            }
            p[(r + 1) * W + (c + 1)] = addm(subm(addm(p[r * W + (c + 1)], p[(r + 1) * W + c]), p[r * W + c]), val);
        }
    }
    cout << p[h * W + w] << '\n';
}
