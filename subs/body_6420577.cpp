
void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}
int r_mod(long a, int m) {
    if (0 <= a && a < m) {
        return int(a);
    } else if (-m <= a && a < 0) {
        return int(a + m);
    }
    a %= m;
    if (a < 0) {
        a += m;
    }
    return int(a);
}
struct HT {
    static constexpr int MAXN = 4 << 20;
    static u64 hsh(u64 x) {
        x += 0x1895618975617926;
        x = (x ^ (x >> 30)) * 0x19856105619025819;
        x = (x ^ (x >> 27)) * 0x18951905612057601;
        return x ^ (x >> 31);
    }
    u64 ks[MAXN + (1 << 16)] {};
    int vs[MAXN + (1 << 16)];
    u64 find(u64 k) {
        u64 i;
        for (i = (hsh(k) & (MAXN - 1)); ks[i] && ks[i] != k; i++)
            ;
        return i;
    }
    bool emplace(u64 k, int v) {
        u64 i = find(k);
        if (!ks[i]) {
            ks[i] = k;
            vs[i] = v;
            return true;
        }
        return false;
    }
    int get(u64 k) {
        return vs[find(k)];
    }
};
vector<int> solve(int kv, const vector<int>& arr, long x) {
    int n = sz(arr);
    vector<int> arr_m[kv];
    for (auto& a : arr) {
        arr_m[r_mod(a, kv)].push_back(a);
    }
    HT from {};
    queue<long> q;
    auto push = [&](long u, int f) -> void {
        bool inserted = from.emplace(u, f);
        if (!inserted) {
            return;
        }
        q.push(u);
    };
    auto push_adj = [&](long u) -> void {
        for (auto& a : arr_m[r_mod(u, kv)]) {
            push((u - a) / kv, a);
        }
    };
    auto gen_answer = [&]() -> vector<int> {
        vector<int> ans;
        long u = 0;
        while (true) {
            int cf = from.get(u);
            ans.push_back(cf);
            u = u * kv + cf;
            if (u == x) {
                break;
            }
        }
        return ans;
    };
    push_adj(x);
    while (sz(q)) {
        long u = q.front();
        q.pop();
        if (!u) {
            return gen_answer();
        }
        push_adj(u);
    }
    return {};
}
void solve() {
    int kv, q, m, maxc;
    cin >> kv >> q >> m >> maxc;
    vector<int> arr(m);
    for (auto& a : arr) {
        cin >> a;
    }
    while (q--) {
        long x;
        cin >> x;
        auto ans = solve(kv, arr, x);
        if (!sz(ans)) {
            cout << "IMPOSSIBLE" << endl;
        } else {
            for (int i = 0; i < sz(ans); i++) {
                cout << ans[i] << " \n"[i == sz(ans) - 1];
            }
        }
    }
}
