#include <cstdio>
#include <set>
int main() {
    std::set<long long> s;
    long long seen = 0, x = 987654321;
    for (int i = 0; i < 2000000; i++) {
        x = (x * 1103515245 + 12345) % 2147483647;
        long long v = x % 1000000;
        if (!s.insert(v).second) seen++;
    }
    long long h = 0;
    for (long long v : s) h = (h * 31 + v) % 1000000007;
    printf("%lld %zu %lld\n", seen, s.size(), h);
}
