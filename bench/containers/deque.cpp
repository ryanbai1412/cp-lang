#include <bits/stdc++.h>
using namespace std;
using ll = long long;

int main() {
    deque<ll> dq;
    for (int i = 0; i < 100000; ++i) {
        dq.push_back(i);
        dq.push_front(i);
    }
    uint64_t rng = 271828182ULL;
    ll acc = 0;
    for (int i = 0; i < 4000000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        int r = (int)((rng >> 33) & 7ULL);
        if (r < 2) {
            dq.push_back(i);
        } else if (r < 4) {
            dq.push_front(i);
        } else if (r < 6) {
            if (!dq.empty()) { acc += dq.front(); dq.pop_front(); }
        } else {
            if (!dq.empty()) { acc += dq.back(); dq.pop_back(); }
        }
    }
    size_t n = dq.size();
    for (int i = 0; i < 2000000; ++i) {
        rng = rng * 6364136223846793005ULL + 1442695040888963407ULL;
        size_t idx = (size_t)((rng >> 33) % n);
        acc += dq[idx];
    }
    cout << acc + (ll)dq.size() << "\n";
}
