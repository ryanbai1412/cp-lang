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

constexpr int hsize = 1 << 22;

int hsh(uint32_t x) {
    x += 0x14121412;
    x ^= x << 17;
    x ^= x >> 11;
    x ^= x << 13;
    return int(x);
}

int v[hsize + int(1e5)][2], m;

void increment(int key) {
    key = hsh(key);
    for (int i = key & (hsize - 1); ; i++) {
        if (v[i][0] == key) {
            v[i][1]++;
            return;
        } else if (!v[i][0]) {
            v[i][0] = key;
            v[i][1] = 1;
            return;
        }
    }
}

int query(int key) {
    key = hsh(key);
    for (int i = key & (hsize - 1); ; i++) {
        if (v[i][0] == key) {
            return v[i][1];
        } else if (!v[i][0]) {
            return 0;
        }
    }
}

int bpow(int base, int exp) {
    int ans = 1;
    while (exp) {
        if (exp & 1) {
            ans *= base;
        }
        base *= base;
        exp >>= 1;
    }
    return ans;
}

template <typename Cb>
void dfs(const vector<pair<int, int>>& arr, int ind, int x, const Cb& cb) {
    if (ind == sz(arr)) {
        cb(x);
        return;
    }
    auto& [c, e] = arr[ind];
    for (int i = 1; i <= m; i++) {
        dfs(arr, ind + 1, x + c * bpow(i, e), cb);
    }
}

void solve() {
    int n;
    cin >> n >> m;
    vector<pair<int, int>> arr(n);
    for (auto& [c, e] : arr) {
        cin >> c >> e;
    }
    vector<int> vl;
    int mid = n / 2;
    dfs({arr.begin(), arr.begin() + mid}, 0, 0, [&](int x) -> void {
        increment(x);
    });
    int ans = 0;
    dfs({arr.begin() + mid, arr.end()}, 0, 0, [&](int x) -> void {
        ans += query(-x);
    });
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}