#include <bits/stdc++.h>
using namespace std;

int main() {
    const int MAX = 200000, n = 200000;
    vector<int> a(n);
    for (int p = 0; p < n; p++) a[p] = 1 + (long long)p * 48271 % MAX;
    long long total = 0;
    for (long long i = 0; i < n; i++) {
        int x = 1 + i * 7919 % MAX;
        int lo = i * 104729 % (n - 1000);
        int c = 0;
        for (int p = lo; p < lo + 1000; p++) c += a[p] % x == 0;
        total += c;
    }
    printf("%lld\n", total);
}
