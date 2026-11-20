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

string dfs(u64 n, u64 k) {
    if (n == 1) {
        assert(k < 2);
        return to_string(k);
    }
    if (k < (u64(1) << (n - 1))) {
        return "0" + dfs(n - 1, k);
    } else {
        u64 p = n < 64 ? u64(1) << n : 0;
        return "1" + dfs(n - 1, p - k - 1);
    }
}

void solve() {
    u64 n, k;
    cin >> n >> k;
    cout << dfs(n, k) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}