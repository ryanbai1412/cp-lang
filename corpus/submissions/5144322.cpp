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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 65;

struct KS {
    int msz;
    vector<ld> v;

    KS(int msz) : msz(msz), v {0.} {}

    void update(const vector<ld>& arr) {
        int n = sz(v), m = sz(arr);
        vector<ld> nv(min(msz, n + m));
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < m && i + j < msz; j++) {
                nv[i + j] = max(nv[i + j], v[i] + arr[j]);
            }
        }
        for (int i = 0; i < sz(nv) - 1; i++) {
            nv[i + 1] = max(nv[i], nv[i + 1]);
        }
        v = nv;
    }
};

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

template <typename T>
vector<T> shiftr(vector<T> arr, int len) {
    arr.insert(arr.begin(), len, 0);
    return arr;
}

template <typename T>
vector<T> vor(vector<T> a, vector<T> b) {
    if (sz(a) < sz(b)) {
        swap(a, b);
    }
    for (int i = 0; i < sz(b); i++) {
        a[i] = max(a[i], b[i]);
    }
    return a;
}

int n, m;
ld kv, cmul, arr[maxn], kpow[maxn];
vector<ld> memo[maxn][maxn];
vector<int> graph[maxn];

vector<ld>& dp(int u, int cdist) {
    auto& ans = memo[u][cdist];
    if (sz(ans)) {
        return ans;
    }
    KS ks(m + 1);
    ks.update({arr[u] * kpow[cdist] * cmul});
    for (auto& v : graph[u]) {
        ks.update(vor(dp(v, cdist + 1), shiftr(dp(v, 1), 1)));
    }
    return ans = ks.v;
}

void solve() {
    cin >> n >> m >> kv;
    int par[n];
    for (auto& a : par) {
        cin >> a;
        a--;
    }
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    vector<int> cyc;
    {
        int u = 0;
        do {
            cyc.push_back(u);
            u = par[u];
        } while (u != 0);
    }
    kpow[0] = 1;
    for (int i = 0; i + 1 < maxn; i++) {
        kpow[i + 1] = kpow[i] * kv;
    }
    ld ans = 0;
    for (int clen = 2; clen <= sz(cyc); clen++) {
        vector<int> ccyc(cyc.begin(), cyc.begin() + clen);
        bool icyc[n] {};
        for (auto& a : ccyc) {
            icyc[a] = true;
        }
        for (int i = 0; i < n; i++) {
            graph[i].clear();
        }
        for (int i = 0; i < n; i++) {
            if (icyc[i]) {
                continue;
            }
            graph[par[i]].push_back(i);
        }
        for (auto& a : memo) {
            for (auto& b : a) {
                b.clear();
            }
        }
        reverse(ccyc.begin() + 1, ccyc.end());
        cmul = 1. / (1 - kpow[clen]);
        KS cans(m + (clen == sz(cyc)));
        for (int i = 0; i < clen; i++) {
            cans.update(dp(ccyc[i], i));
        }
        ans = max(ans, cans.v.back());
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(2);
    solve();
}