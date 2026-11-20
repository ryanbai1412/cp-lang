
struct IO {
    char buf[20 << 20], *i = buf;
    IO() {
        fread_unlocked(buf, 1, sizeof(buf), stdin);
    }
    IO& operator>>(int& x) {
        x = 0;
        while (*i >= '0') {
            x = x * 10 + (*(i++) ^ '0');
        }
        i++;
        return *this;
    }
} io;
struct DS {
    int n;
    vector<int> pl, pr;
    DS(int n) : n(n), pl(n), pr(n) {
        iota(begin(pl), end(pl), -1);
        iota(begin(pr), end(pr), 1);
    }
    void insert(int u) {
        pl[u] = pr[u] = u;
    }
    int query_l(int u) {
        if (u == -1 || pl[u] == u) {
            return u;
        }
        return pl[u] = query_l(pl[u]);
    }
    int query_r(int u) {
        if (u == sz(pl) || pr[u] == u) {
            return u;
        }
        return pr[u] = query_r(pr[u]);
    }
    int query_l_clamped(int u) {
        return query_l(clamp(u, -1, n - 1));
    }
    int query_r_clamped(int u) {
        return query_r(clamp(u, 0, n));
    }
    void erase(int u) {
        pl[u] = u - 1;
        pr[u] = u + 1;
    }
};
template <typename Cb>
void solve(const vector<int>& arr, bool prefer_first, const Cb& cb) {
    int n = sz(arr), m = *max_element(begin(arr), end(arr)) + 1;
    DS ds(m);
    for (auto& a : arr) {
        ds.insert(a);
    }
    int rep[m], cur_inds[m], next_ind[m];
    memset(rep, -1, sizeof(rep));
    memset(cur_inds, -1, sizeof(cur_inds));
    for (int i = n - 1; i >= 0; i--) {
        next_ind[i] = cur_inds[arr[i]];
        cur_inds[arr[i]] = i;
        if (prefer_first || rep[arr[i]] == -1) {
            rep[arr[i]] = i;
        }
    }
    long sum_l = 0, sum_r = accumulate(begin(arr), end(arr), long(0));
    for (int i = 0; i < n; i++) {
        sum_r -= arr[i];
        if (prefer_first) {
            rep[arr[i]] = next_ind[i];
        } else if (rep[arr[i]] == i) {
            rep[arr[i]] = -1;
        }
        if (rep[arr[i]] == -1) {
            ds.erase(arr[i]);
        }
        cb(abs(sum_l - sum_r), i, -1);
        long diff = sum_r - sum_l;
        if (0 <= diff && diff <= int(1e9)) {
            auto it = ds.query_r_clamped(diff / 2);
            auto go = [&](int x) -> void { cb(abs(diff - 2 * x), i, rep[x]); };
            if (it < m) {
                go(it);
            }
            it = ds.query_l_clamped(it - 1);
            if (it >= 0) {
                go(it);
            }
        }
        sum_l += arr[i];
    }
}
void solve() {
    int n;
    io >> n;
    vector<int> arr(n);
    for (auto& a : arr) {
        io >> a;
    }
    tuple<long, int, int> opt {long(1e18), -1, -1};
    solve(arr, true, [&](long x, int i1, int i2) -> void {
        opt = min(opt, {x, i1, i2});
    });
    reverse(begin(arr), end(arr));
    solve(arr, false, [&](long x, int i1, int i2) -> void {
        if (i2 == -1) {
            return;
        }
        opt = min(opt, {x, n - i1 - 1, n - i2 - 1});
    });
    auto& [ans, i1, i2] = opt;
    dbg(ans);
    cout << i1 + 1 << " " << (i2 == -1 ? -1 : i2 + 1) << endl;
}
