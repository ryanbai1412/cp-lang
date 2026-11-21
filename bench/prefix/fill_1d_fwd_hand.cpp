#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

static constexpr uint32_t MOD = 998244353;

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int64_t n, seed;
    if (!(cin >> n >> seed)) return 0;
    vector<uint32_t> p(n + 1, 0);
    int64_t s = seed;
    for (int64_t i = 0; i < n; ++i) {
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        uint32_t val = 1;
        if (i > 0) {
            int64_t lo = s & (i - 1);
            uint32_t diff = p[i] >= p[lo] ? p[i] - p[lo] : p[i] + MOD - p[lo];
            val = diff + 1;
            if (val >= MOD) val -= MOD;
        }
        uint32_t nx = p[i] + val;
        if (nx >= MOD) nx -= MOD;
        p[i + 1] = nx;
    }
    cout << p[n] << '\n';
}
