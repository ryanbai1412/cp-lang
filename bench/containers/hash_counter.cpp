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
    long long x = 987654321;
    for (int i = 0; i < 4000000; i++) {
        x = (x * 1103515245LL + 12345LL) % 2147483647LL;
        auto it = m.insert({x % 1000000, 0}).first;
        it->second += i % 4 == 3 ? -1 : 1;
        if (it->second == 0) m.erase(it->first);
    }
    cout << m.size() << '\n';
}
