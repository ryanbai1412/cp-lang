#include <algorithm>
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int64_t n, q, seed;
    if (!(cin >> n >> q >> seed)) return 0;
    int64_t mask = n - 1;
    int64_t s = seed;
    vector<int64_t> a(n);
    for (int64_t i = 0; i < n; ++i) {
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        a[i] = (s & 2047) - 1024;
    }
    vector<int64_t> p(n + 1, 0);
    for (int64_t i = 0; i < n; ++i) p[i + 1] = p[i] + a[i];
    int64_t acc = 0;
    for (int64_t k = 0; k < q; ++k) {
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t x = s & mask;
        s = (s * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t y = s & mask;
        int64_t l = min(x, y), r = max(x, y);
        acc += p[r + 1] - p[l];
    }
    cout << acc << '\n';
}
