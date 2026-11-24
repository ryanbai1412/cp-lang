#include <bits/stdc++.h>
#include <ext/pb_ds/assoc_container.hpp>
using namespace std;

int main() {
    const int n = 300000;
    __gnu_pbds::gp_hash_table<string, long long> m;
    long long x = 987654321;
    for (int i = 0; i < n; i++) {
        x = (x * 1103515245LL + 12345LL) % 2147483647LL;
        m["k" + to_string(x)] = i;
    }
    long long sum = 0;
    x = 987654321;
    for (int i = 0; i < n; i++) {
        x = (x * 1103515245LL + 12345LL) % 2147483647LL;
        auto it = m.find((i % 2 == 0 ? "k" : "m") + to_string(x));
        if (it != m.end()) sum += it->second;
    }
    printf("%d %lld\n", (int)m.size(), sum);
}
