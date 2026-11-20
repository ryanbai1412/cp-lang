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

using u64 = uint64_t;

tuple<u64, u64, u64> egcd(u64 a, u64 b) {
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
    u64 ans = get<1>(egcd(x, m));
    if (ans >= m) {
        ans = -ans + 1;
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}