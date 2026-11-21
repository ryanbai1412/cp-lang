#include <bits/stdc++.h>
#include <ext/pb_ds/assoc_container.hpp>
using namespace std;

struct SplitMixHash {
    static uint64_t splitmix64(uint64_t x) {
        x += 0x9e3779b97f4a7c15ULL;
        x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
        x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
        return x ^ (x >> 31);
    }
    size_t operator()(uint64_t x) const {
        static const uint64_t FIXED_RANDOM = chrono::steady_clock::now().time_since_epoch().count();
        return splitmix64(x + FIXED_RANDOM);
    }
};

int main() {
    __gnu_pbds::gp_hash_table<long long, long long, SplitMixHash> m;
    for (long long i = 1; i <= 150000; i++) {
        m[i * 126271] = i;
        m[i * 64] = i + 1;
    }
    long long sum = 0;
    for (long long i = 1; i <= 150000; i++) {
        auto it1 = m.find(i * 126271);
        auto it2 = m.find(i * 64);
        sum += (it1 != m.end() ? it1->second : 0) + (it2 != m.end() ? it2->second : 0);
    }
    cout << sum << '\n';
}
