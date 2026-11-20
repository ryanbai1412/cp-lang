#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(T&& t, U&&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

void solve() {
    u64 x;
    cin >> x;
    u64 ans = 1;
    for (u64 i = 1; i <= x && ans; i++) {
        ans *= i;
    }
    cout << ((ans) & ((u64(1) << 32) - 1)) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}