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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

constexpr int maxn = 1e6, logn = 21;

int v[4 << logn];

void update(int o, int depth, int ind, int x) {
    if (depth == -1) {
        v[o] = max(v[o], x);
        return;
    }
    int c[2] {o * 2, o * 2 + 1};
    update(c[on(ind, depth)], depth - 1, ind, x);
    v[o] = max(v[c[0]], v[c[1]]);
}

void update(int ind, int x) {
    update(1, logn, ind, x);
}

int query(int o, int depth, int m, int kv) {
    if (depth == -1) {
        return v[o];
    }
    int c[2] {o * 2, o * 2 + 1}, chi = on(m, depth) ^ on(kv, depth);
    int ans = query(c[chi], depth - 1, m, kv);
    if (on(kv, depth)) {
        ans = max(ans, v[c[!chi]]);
    }
    return ans;
}

int query(int m, int kv) {
    return query(1, logn, m, kv);
}

void solve() {
    int n, q;
    cin >> n >> q;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    for (int i = 0; i < n; i++) {
        update(i, arr[i]);
    }
    while (q--) {
        int m, kv;
        cin >> m >> kv;
        cout << query(m, kv) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}