// The AC submission corpus/submissions/3657448.cpp (a grader function) with a grader main
// that reads the stdin adaptation: `N S`, the N arguments are splitmix64(S) outputs, and the
// answer is the sum of setbits over them.
#include "bits/stdc++.h"

int setbits(unsigned long long x) {
    return __builtin_popcountll(x);
}

int main() {
    unsigned long long n, s;
    std::cin >> n >> s;
    unsigned long long total = 0;
    for (unsigned long long i = 0; i < n; i++) {
        unsigned long long z = (s += 0x9e3779b97f4a7c15ULL);
        z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
        z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
        total += setbits(z ^ (z >> 31));
    }
    std::cout << total << "\n";
}
