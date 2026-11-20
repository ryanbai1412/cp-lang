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

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 1e5 + 5;

struct H {
    int id = 0;
    map<vector<u64>, u64> m;
    
    u64 operator()(const vector<u64>& arr) {
//        u64 ans = 0x1412;
//        for (auto& a : arr) {
//            ans ^= ans << 21;
//            ans ^= ans >> 33;
//            ans ^= ans << 19;
//            ans += a;
//        }
        auto [it, inserted] = m.emplace(arr, id);
        id += inserted;
        return it->second;
    }
} ahash;

int root[2], siz[2][maxn];
u64 hsh[2][maxn];
vector<int> graph[2][maxn];
map<pair<int, int>, bool> memo;

void pdfs(int g, int u) {
    siz[g][u] = 1;
    for (auto& v : graph[g][u]) {
        pdfs(g, v);
        siz[g][u] += siz[g][v];
    }
}

u64 dfs1(int g, int u) {
    vector<u64> ch;
    for (auto& v : graph[g][u]) {
        ch.push_back(dfs1(g, v));
    }
    sort(begin(ch), end(ch));
    return hsh[g][u] = ahash(ch);
}

int estimate(int u1, int u2) {
    int s1 = siz[0][u1];
    if (u2 == -1) {
        return s1;
    }
    int s2 = siz[1][u2];
    if (s1 < s2 || sz(graph[0][u1]) < sz(graph[1][u2])) {
        return 10;
    }
    return s1 - s2;
}

bool dp(int u1, int u2) {
    if (u2 == -1) {
        return true;
    }
    u64 h1 = hsh[0][u1], h2 = hsh[1][u2];
    if (h1 == h2) {
        return true;
    }
    int est = estimate(u1, u2);
    assert(est <= 5);
    auto [it, inserted] = memo.insert({{h1, h2}, false});
    if (!inserted) {
        return it->second;
    }
    vector<pair<u64, int>> ch1, ch2;
    map<u64, int> cnt;
    for (auto& v : graph[0][u1]) {
        u64 h = hsh[0][v];
        ch1.emplace_back(h, v);
        cnt[h]++;
    }
    for (auto& v : graph[1][u2]) {
        u64 h = hsh[1][v];
        ch2.emplace_back(h, v);
        cnt[h]--;
    }
    sort(begin(ch1), end(ch1));
    sort(begin(ch2), end(ch2));
    vector<int> rep1, rep2;
    for (auto& [k, v] : cnt) {
        if (v < 0) {
            int x = lower_bound(begin(ch2), end(ch2), pair<u64, int> {k, 0})
                        ->second;
            for (int i = 0; i < -v; i++) {
                rep2.push_back(x);
            }
        } else {
            int x = lower_bound(begin(ch1), end(ch1), pair<u64, int> {k, 0})
                        ->second;
            for (int i = 0; i < v; i++) {
                rep1.push_back(x);
            }
        }
    }
    assert(sz(rep1) >= sz(rep2));
    if (sz(rep1) > est) {
        return it->second = false;
    }
    rep2.resize(sz(rep1), -1);
    sort(begin(rep2), end(rep2));
    do {
        int cest = 0;
        for (int i = 0; i < sz(rep1); i++) {
            cest += estimate(rep1[i], rep2[i]);
        }
        if (cest > est) {
            continue;
        }
        for (int i = 0; i < sz(rep1); i++) {
            if (!dp(rep1[i], rep2[i])) {
                goto loop;
            }
        }
        return it->second = true;
    loop:;
    } while (next_permutation(begin(rep2), end(rep2)));
    return it->second = false;
}

void solve() {
    auto read_graph = [&](int g) -> void {
        int n;
        cin >> n;
        for (int i = 0; i < n; i++) {
            graph[g][i].clear();
        }
        for (int i = 0; i < n; i++) {
            int p;
            cin >> p;
            if (p == -1) {
                root[g] = i;
                continue;
            }
            p--;
            graph[g][p].push_back(i);
        }
    };
    for (int i = 0; i < 2; i++) {
        read_graph(i);
        pdfs(i, root[i]);
        dfs1(i, root[i]);
    }
    memo.clear();
    if (estimate(root[0], root[1]) <= 5 && dp(root[0], root[1])) {
        cout << "Yes" << endl;
    } else {
        cout << "No" << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    int tc_id, tcs, k;
    cin >> tc_id >> tcs >> k;
    while (tcs--) {
        solve();
    }
}