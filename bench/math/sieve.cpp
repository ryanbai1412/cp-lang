// Matched C++17 reference for bench/math/sieve.cp (same linear sieve + uint32_t spf/primes + ll phi/mu tables).
#include <cstdint>
#include <iostream>
#include <vector>
using ll = long long;

int main() {
    const int n = 5000000;
    std::vector<uint32_t> spf(n + 1, 0);
    std::vector<uint32_t> primes;
    for (int i = 2; i <= n; ++i) {
        if (spf[i] == 0) {
            spf[i] = uint32_t(i);
            primes.push_back(uint32_t(i));
        }
        for (uint32_t p : primes) {
            if (p > spf[i] || ll(i) * ll(p) > n) break;
            spf[i * int(p)] = p;
        }
    }
    std::vector<ll> phi(n + 1, 0), mu(n + 1, 0);
    phi[1] = 1;
    mu[1] = 1;
    for (int i = 2; i <= n; ++i) {
        int p = int(spf[i]);
        int q = i / p;
        phi[i] = (int(spf[q]) == p) ? phi[q] * p : phi[q] * (p - 1);
    }
    for (int i = 2; i <= n; ++i) {
        int p = int(spf[i]);
        int q = i / p;
        mu[i] = (int(spf[q]) == p) ? 0 : -mu[q];
    }
    ll acc = 0;
    for (int i = 1; i <= n; ++i) acc += phi[i] + mu[i] * i;
    std::cout << primes.size() << ' ' << acc << '\n';
}
