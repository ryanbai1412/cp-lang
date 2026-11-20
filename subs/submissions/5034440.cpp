#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
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

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

using u64 = __uint64_t;
using i128 = __int128;

tuple<i128, i128, i128> egcd(i128 a, i128 b) {
    if (!a) {
        return {b, 0, 1};
    } else {
        auto [g, x, y] = egcd(b % a, a);
        return {g, y - (b / a) * x, x};
    }
}

void solve() {
    u64 x, m;
    cin >> x >> m;
    i128 ans = get<1>(egcd(x, m));
    if (ans < 0) {
        ans += m;
    }
    cout << u64(ans) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}