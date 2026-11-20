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

template <typename T>
T sorted(T arr) {
    sort(begin(arr), end(arr));
    return arr;
}

constexpr long mod = 1e9 + 7;

struct Hasher {
    vector<long> psum, bpow;

    Hasher(const vector<int>& arr, long base) {
        bpow.push_back(1);
        psum.push_back(0);
        for (auto& a : arr) {
            bpow.push_back((bpow.back() * base) % mod);
            psum.push_back((psum.back() * base + a) % mod);
        }
    }

    int query(int l, int r) const {
        long ans = (psum[r] - psum[l] * bpow[r - l]) % mod;
        if (ans < 0) {
            ans += mod;
        }
        return int(ans);
    }
};

void solve() {
    int n, _m;
    cin >> n >> _m;
    vector<int> arra(n), arrb(n);
    for (auto& a : arra) {
        cin >> a;
    }
    for (auto& a : arrb) {
        cin >> a;
    }
    vector<Hasher> hsha, hshb;
    for (int i = 0; i < 3; i++) {
        hsha.emplace_back(arra, 14121412 + i);
        hshb.emplace_back(arrb, 14121412 + i);
    }
    auto query = [&](const vector<Hasher>& h, int l, int r) -> array<int, 3> {
        array<int, 3> ans;
        for (int i = 0; i < 3; i++) {
            ans[i] = h[i].query(l, r);
        }
        return ans;
    };
    for (int i = 1; i < n; i++) {
        for (int j = i + 1; j < n; j++) {
            array<array<int, 3>, 3> hsa = {query(hsha, 0, i), query(hsha, i, j),
                                           query(hsha, j, n)};
            array<array<int, 3>, 3> hsb = {query(hshb, 0, i), query(hshb, i, j),
                                           query(hshb, j, n)};
            if (sorted(hsa) != sorted(hsb)) {
                continue;
            }
            dbg(i, hsa[0] == hsb[0]);
            cout << "YES" << endl;
            bool vis[3] {};
            for (auto& a : hsa) {
                for (int b = 0; b < 3; b++) {
                    if (vis[b] || hsb[b] != a) {
                        continue;
                    }
                    vis[b] = true;
                    if (b == 0) {
                        cout << 1 << " " << i << endl;
                    } else if (b == 1) {
                        cout << i + 1 << " " << j << endl;
                    } else {
                        cout << j + 1 << " " << n << endl;
                    }
                }
            }
            return;
        }
    }
    cout << "NO" << endl;
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