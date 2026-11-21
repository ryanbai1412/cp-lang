#include <bits/stdc++.h>
using namespace std;
using ll = long long;

int main() {
    priority_queue<ll, vector<ll>, greater<ll>> h;
    uint64_t rng = 314159265ULL;
    for (int i = 0; i < 500000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        h.push((ll)((rng >> 33) % 1000000000ULL));
    }
    ll acc = 0;
    for (int i = 0; i < 500000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        h.push((ll)((rng >> 33) % 1000000000ULL));
        acc += h.top();
        h.pop();
    }
    while (!h.empty()) {
        acc ^= h.top();
        h.pop();
    }
    cout << acc << "\n";
}
