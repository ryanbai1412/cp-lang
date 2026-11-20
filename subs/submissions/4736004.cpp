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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

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

int use_machine(std::vector<int> x);

const int bsize = 100;

template <typename T>
struct DT {
    vector<int> query;
    vector<T> res;

    operator bool() const {
        return sz(query);
    }
};

template <typename U, typename T>
U feach(int n, int x, int y, const T& t) {
    vector<int> perm(n);
    iota(begin(perm), end(perm), 2);
    perm.insert(perm.end(), x, 0);
    perm.insert(perm.end(), y, 1);
    do {
        U cur = t(perm);
        if (cur) {
            return cur;
        }
    } while (next_permutation(begin(perm), end(perm)));
    return U {};
}

int eval(int mask, const vector<int>& perm) {
    auto get = [&](int ind) -> int {
        if (perm[ind] >= 2) {
            return (mask >> (perm[ind] - 2)) & 1;
        }
        return perm[ind];
    };
    int ans = 0;
    for (int i = 0; i < sz(perm) - 1; i++) {
        ans += get(i) != get(i + 1);
    }
    return ans;
}

// n unknowns, x zeroes, y ones
DT<DT<int>> compute_dt(int n, int x, int y) {
    int tot = n + x + y;
    return feach<DT<DT<int>>>(n, x, y, [&](const vector<int>& perm) {
        vector<int> res[tot];
        for (int i = 0; i < (1 << n); i++) {
            res[eval(i, perm)].push_back(i);
        }
        DT<DT<int>> ans;
        ans.query = perm;
        for (auto& a : res) {
            DT<int> cur = feach<DT<int>>(n, x, y, [&](const vector<int>& perm) {
                DT<int> ans2;
                ans2.query = perm;
                ans2.res.resize(tot, -1);
                for (auto& b : a) {
                    int cur = eval(b, perm);
                    if (ans2.res[cur] != -1) {
                        return DT<int> {};
                    }
                    ans2.res[cur] = b;
                }
                return ans2;
            });
            if (!cur) {
                return DT<DT<int>> {};
            }
            ans.res.push_back(cur);
        }
        return ans;
    });
}

struct State {
    int i = 1, ans = 0, n;
    vector<int> inds[2];
    map<array<int, 3>, DT<DT<int>>> cache;

    State(int n) : n(n), inds {{0}, {}} {}

    int query(const vector<int>& arr, const vector<int>& vars) {
        int cind[2] {};
        vector<int> cur;
        for (auto& a : arr) {
            if (a < 2) {
                cur.push_back(inds[a][cind[a]++]);
            } else {
                cur.push_back(vars[a - 2]);
            }
        }
        dbg(arr, cur);
        return use_machine(cur);
    }

    void eval(int n, int x, int y) {
        n = min(n, this->n - i);
        auto [it, inserted] = cache.insert({{n, x, y}, {}});
        if (inserted) {
            it->second = compute_dt(n, x, y);
        }
        vector<int> vars;
        for (int i = 0; i < n; i++) {
            vars.push_back(this->i++);
        }
        eval(it->second, vars);
    }

    void eval(const DT<DT<int>>& dt, const vector<int>& vars) {
        eval(dt.res[query(dt.query, vars)], vars);
    }

    void eval(const DT<int>& dt, const vector<int>& vars) {
        vector<int> poss;
        for (int i = 0; i < sz(dt.res); i++) {
            if (dt.res[i] != -1) {
                poss.push_back(dt.res[i]);
            }
        }
        int ans;
        if (sz(poss) == 1) {
            ans = poss[0];
        } else {
            ans = dt.res[query(dt.query, vars)];
        }
        for (int i = 0; i < sz(vars); i++) {
            inds[(ans >> i) & 1].push_back(vars[i]);
        }
    }

    void eval_opt() {
        int a = sz(inds[0]), b = sz(inds[1]);
        if (a == 1 && b == 0) {
            eval(1, 1, 0);
        } else if (a + b == 2) {
            eval(2, a, b);
        } else {
            assert(a + b >= 4);
            if (a && b) {
                int ax = min(a, 3), bx = 4 - ax;
                assert(ax <= a && bx <= b);
                eval(5, ax, bx);
            } else {
                assert(a >= 4 && !b);
                eval(4, 4, 0);
            }
        }
    }

    void query_mult() {
        int cind;
        if (sz(inds[0]) >= sz(inds[1])) {
            cind = 0;
        } else {
            cind = 1;
        }
        vector<int> cur;
        for (int j = 0; i < n && j < sz(inds[cind]); i++, j++) {
            cur.push_back(inds[cind][j]);
            cur.push_back(i);
        }
        int cans = use_machine(cur), x = (cans + 1) / 2;
        if (!cind) {
            ans += sz(cur) / 2 - x;
        } else {
            ans += x;
        }
        if (cans & 1) {
            inds[cind ^ 1].push_back(cur.back());
        } else {
            inds[cind].push_back(cur.back());
        }
    }

    int solve() {
        ans = sz(inds[0]);
        while (i < n) {
            query_mult();
        }
        return ans;
    }
};

int count_mushrooms(int n) {
    State state(n);
    while (state.i < n && state.i < 2 * bsize - 1) {
        state.eval_opt();
        dbg(sz(state.inds[0]), sz(state.inds[1]));
    }
    return state.solve();
}