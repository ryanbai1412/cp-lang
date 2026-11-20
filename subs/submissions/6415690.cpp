#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

long cdiv(long a, long b) {
    if (b < 0) {
        a = -a;
        b = -b;
    }
    if (a < 0) {
        return a / b;
    } else {
        return (a + b - 1) / b;
    }
}

struct Line {
    long m, b;

    long operator()(long x) const {
        return m * x + b;
    }
};

long inter(const Line& l1, const Line& l2) {
    assert(l1.m > l2.m);
    return cdiv(l2.b - l1.b, l1.m - l2.m);
}

struct DS1 {
    long x;
    vector<pair<long, Line>> lines;
    int li = 0;

    DS1() {}
    DS1(long x, const vector<Line>& arr) : x(x) {
        for (auto& a : arr) {
            while (sz(lines) && a(lines.back().first) >
                   lines.back().second(lines.back().first)) {
                lines.pop_back();
            }

            if (!sz(lines)) {
                lines.emplace_back(0, a);
            } else {
                lines.emplace_back(inter(a, lines.back().second), a);
            }
        }
    }

    void upd_x() {
        x++;
    }

    long query() {
        while (li + 1 < sz(lines) && x >= lines[li + 1].first) {
            li++;
        }
        return lines[li].second(x);
    }
};

constexpr int BSIZE = 400;

struct Block {
    int n, add = 0;
    vector<int> arr, v;
    DS1 ds1;

    Block(const vector<int>& arr) : n(sz(arr)), arr(arr), v(n) {
        rebuild();
    }

    void rebuild() {
        vector<Line> lines;
        for (int i = 0; i < n; i++) {
            lines.push_back({arr[i], long(arr[i]) * v[i]});
        }
        ds1 = DS1(add, lines);
    }

    void update_all() {
        add++;
        ds1.upd_x();
    }

    void update_pref(int ind) {
        for (int i = 0; i < ind; i++) {
            v[i]++;
        }
        rebuild();
    }

    long query() {
        return ds1.query();
    }
};

struct DS {
    int n;
    vector<int> arr, v;
    vector<Block> blocks;

    DS(const vector<int>& arr) : n(sz(arr)), arr(arr), v(n) {
        for (int ql = 0; ql < n; ql += BSIZE) {
            int qr = min(n, ql + BSIZE);
            blocks.emplace_back(vector(begin(arr) + ql, begin(arr) + qr));
        }
    }

    void update(int ind) {
        int bind = ind / BSIZE;
        for (int i = 0; i < bind; i++) {
            blocks[i].update_all();
        }
        if (ind % BSIZE) {
            blocks[bind].update_pref(ind - bind * BSIZE);
        }
    }

    long query() {
        long mx = 0;
        for (auto& a : blocks) {
            mx = max(mx, a.query());
        }
        return mx;
    }
};

void solve() {
    int n;
    cin >> n;
    vector<pair<int, int>> arr(n);
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }

    map<int, vector<int>> evts;
    for (int i = 0; i < n; i++) {
        evts[arr[i].first].push_back(i);
    }

    vector<int> ys = [&]() {
        vector<int> res;
        for (auto& [_x, y] : arr) {
            res.push_back(y);
        }
        sort(begin(res), end(res));
        res.erase(unique(begin(res), end(res)), end(res));

        return res;
    }();

    DS ds(ys);
    int gt_rem = n;
    long ans = 0;

    for (auto& [k, ev] : evts) {
        ans = max(ans, gt_rem * long(k) + ds.query());

        for (auto& i : ev) {
            ds.update(int(upper_bound(begin(ys), end(ys), arr[i].second) -
                          begin(ys)));
        }
        gt_rem -= sz(ev);
    }

    ans = max(ans, ds.query());

    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}