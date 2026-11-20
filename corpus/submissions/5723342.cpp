#include "bits/extc++.h"
     
    using namespace std;
     
    template <typename T, typename... U>
    void dbgh(const T& t, const U&... u) {
        cerr << t;
        ((cerr << " | " << u), ...);
        cerr << endl;
    }
     
    #ifdef DEBUG
    #define dbg(...)                                              \
        cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
        dbgh(__VA_ARGS__)
    #else
    #define dbg(...)
    #define cerr   \
        if (false) \
        cerr
    #endif
     
    #define endl "\n"
    #define long int64_t
    #define sz(x) int(std::size(x))
     
    struct DSU {
        vector<int> p;
     
        DSU(int n) : p(n, -1) {}
     
        int find(int u) {
            return p[u] < 0 ? u : (p[u] = find(p[u]));
        }
     
        int size(int u) {
            return -p[find(u)];
        }
     
        bool merge(int u, int v) {
            u = find(u);
            v = find(v);
            if (u == v) {
                return false;
            }
            if (p[u] < p[v]) {
                swap(u, v);
            }
            p[v] += p[u];
            p[u] = v;
            return true;
        }
    };
     
    int getDistance(int i, int j);
     
    struct G {
        map<pair<int, int>, int> cache;
     
        int query(int u, int v) {
            if (u == v) {
                return 0;
            }
            if (u > v) {
                swap(u, v);
            }
            auto [it, inserted] = cache.insert({{u, v}, -1});
            if (!inserted) {
                return it->second;
            }
            return it->second = getDistance(u, v);
        }
    } G;
     
    int hubDistance(int n, int tc_id) {
        G = {};
     
        auto farthest = [&](int u) -> int {
            pair<int, int> ans {-1, 0};
            for (int i = 0; i < n; i++) {
                ans = max(ans, {G.query(u, i), i});
            }
            return ans.second;
        };
     
        int u1 = farthest(0), u2 = farthest(u1), diam = G.query(u1, u2);
     
        int u_hori[n], u_down[n];
        for (int i : {0}) {
            int d1 = G.query(u1, i), d2 = G.query(u2, i);
     
            u_down[i] = (d1 + d2 - diam) / 2;
            u_hori[i] = d1 - u_down[i];
        }
        for (int i = 1; i < n; i++) {
            int d1 = G.query(u1, i), d0 = G.query(0, i);
     
            int common = (d1 + d0 - G.query(0, u1)) / 2, l0 = d0 - common, l1 = d1 - common;
            if (l0 <= u_down[0]) {
                u_hori[i] = u_hori[0];
                u_down[i] = common + l1 - u_hori[0];
            } else if (l1 < u_hori[0]) {
                u_hori[i] = l1;
                u_down[i] = common;
            } else {
                assert(l1 > u_hori[0]);
                u_hori[i] = d1;
                u_down[i] = 0;
            }
        }
     
        int ans = 1e9;
     
        for (int i = 0; i < n; i++) {
            ans = min(ans, max(u_hori[i], diam - u_hori[i]));
        }
     
        vector<int> cands;
        {
            map<int, int> mp;
            for (auto& a : u_hori) {
                mp[a]++;
            }
     
            bool ok = false;
            int cl = 0;
            for (auto& [k, v] : mp) {
                if (max(k, diam - k) > ans) {
                    cl += v;
                    continue;
                }
     
                int cm = v, cr = n - cl - cm;
                dbg(cl, cm, cr);
                if (max({cl, cm, cr}) <= n / 2) {
                    ok = true;
                }
                if (max(cl, cr) <= n / 2) {
                    cands.push_back(k);
                }
     
                cl += v;
            }
     
            if (ok) {
                return ans;
            }
        }
     
        if (!sz(cands)) {
            return -ans;
        }
        assert(sz(cands) == 1);
     
        if (tc_id > 2 && tc_id != 4) {
            vector<int> nodes;
            for (int i = 0; i < n; i++) {
                if (u_hori[i] == cands[0]) {
                    nodes.push_back(i);
                }
            }
     
            auto same_child = [&](int u, int v) -> bool {
                return G.query(u, v) != u_down[u] + u_down[v];
            };
     
            vector<int> u_same, u_diff;
            vector<pair<vector<int>, vector<int>>> segs;
     
            for (auto& a : nodes) {
                if (!sz(u_same)) {
                    u_same.push_back(a);
                } else {
                    if (same_child(u_same[0], a)) {
                        u_same.push_back(a);
                    } else {
                        u_diff.push_back(a);
                    }
                }
     
                if (sz(u_same) == sz(u_diff)) {
                    segs.emplace_back(u_same, u_diff);
                    u_same.clear();
                    u_diff.clear();
                }
            }
     
            if (!sz(u_same)) {
                return ans;
            }
     
            int ccnt = sz(u_same);
            for (auto& [s, d] : segs) {
                if (same_child(s[0], u_same[0])) {
                    ccnt += sz(d);
                }
     
                for (auto& a : d) {
                    if (same_child(a, u_same[0])) {
                        ccnt++;
                    }
                }
            }
     
            if (ccnt <= n / 2) {
                return ans;
            }
     
            return -ans;
        }
     
        return -ans;
    }