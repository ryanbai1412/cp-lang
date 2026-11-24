#include <cstdint>
#include <iostream>
#include <vector>

int main() {
    const int n = 100000;
    std::vector<int64_t> v;
    int64_t seed = 987654321;
    int64_t acc = 0;
    for (int i = 0; i < n; i++) {
        seed = (seed * 1103515245LL + 12345LL) % 2147483647LL;
        v.insert(v.begin(), seed % 1000003);
        if (i % 4 == 3) {
            size_t at = v.size() / 2;
            acc += v[at];
            v.erase(v.begin() + at);
        }
    }
    acc += v[0] + v[v.size() - 1] + (int64_t)v.size();
    std::cout << acc << "\n";
}
