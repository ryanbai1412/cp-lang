#include <algorithm>
#include <iostream>
#include <vector>

using ll = long long;

int main() {
    const int n = 200000;
    std::vector<ll> base(n), xs(n);
    for (int i = 0; i < n; i++) {
        base[i] = ((ll)i * 1103515245LL + 12345LL) % 2147483647LL;
    }
    ll acc = 0;
    for (int r = 0; r < 80; r++) {
        ll mask = ((ll)r * 2654435761LL) % 2147483647LL;
        for (int i = 0; i < n; i++) xs[i] = base[i] ^ mask;
        int l = (r * 3713) % (n / 2);
        int hi = l + n / 2;
        int k = (r * 1009) % (n / 2);
        std::nth_element(xs.begin() + l, xs.begin() + l + k, xs.begin() + hi);
        acc += xs[l + k];
    }
    std::cout << acc << '\n';
}
