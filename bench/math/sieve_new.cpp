// Matched C++17 reference for bench/math/sieve_new.cp (same linear sieve, uint32_t spf/primes).
#include <cstdint>
#include <iostream>
#include <vector>
using ll = long long;

int main() {
    const int n = 1000000;
    ll acc = 0;
    for (int r = 0; r < 20; ++r) {
        std::vector<uint32_t> spf(n + 1, 0);
        std::vector<uint32_t> primes;
        for (int i = 2; i <= n; ++i) {
            if (spf[i] == 0) {
                spf[i] = uint32_t(i);
                primes.push_back(uint32_t(i));
            }
            uint32_t si = spf[i];
            for (uint32_t p : primes) {
                if (p > si || ll(i) * ll(p) > n) break;
                spf[i * int(p)] = p;
            }
        }
        acc += ll(primes.size()) + ll(spf[n - r]);
    }
    std::cout << acc << '\n';
}
