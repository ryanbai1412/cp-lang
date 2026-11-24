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

void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

template <typename T>
T sorted(T arr) {
    sort(begin(arr), end(arr));
    return arr;
}

mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());

long rint(long l, long r) {
    return cowng() % (r - l + 1) + l;
}

vector<int> tally(const vector<int>& arr) {
    int n = sz(arr);

    vector<int> ans(n);
    for (auto& a : arr) {
        ans[a]++;
    }

    return ans;
}

bool solve12(const vector<int>& arra, const vector<int>& arrb) {
    int n = sz(arra);

    if (*max_element(begin(arra), end(arra)) > 1 ||
        *max_element(begin(arrb), end(arrb)) > 1) {
        return false;
    }

    if (arra == reversed(arrb)) {
        cout << 1 << endl;
        for (int it = 0; it < 2; it++) {
            for (int i = 0; i < n; i++) {
                cout << 1 << " \n"[i == n - 1];
            }
        }
        return true;
    }

    cout << 2 << endl;
    for (auto& cv : {arra, arrb}) {
        for (int i = 0; i < n; i++) {
            cout << cv[i] + 1 << " \n"[i == n - 1];
        }
    }

    return true;
}

bool solve_dist(vector<int> arra, const vector<int>& arrb) {
    int n = sz(arra);
    auto init_arra = arra;
    reverse(begin(arra), end(arra));

    auto is_perm = [&](const vector<int>& arr) -> bool {
        vector<bool> vis(n);

        for (auto& a : arr) {
            if (vis[a]) {
                return false;
            }
            vis[a] = true;
        }

        return true;
    };

    if (!is_perm(arra) || !is_perm(arrb)) {
        return false;
    }

    vector<int> iarra(n);
    for (int i = 0; i < n; i++) {
        iarra[arra[i]] = i;
    }

    vector<int> cf(n), ans_id(n);
    auto cf_l = end(cf);

    for (auto& a : arrb) {
        int cx = iarra[a];

        auto it = --lower_bound(cf_l, end(cf), cx);
        *it = cx;
        dbg(a, end(cf) - it);
        ans_id[a] = int(end(cf) - it);

        if (it == prev(cf_l)) {
            cf_l = it;
        }
    }

    cout << end(cf) - cf_l << endl;
    for (auto& cv : {init_arra, arrb}) {
        for (int i = 0; i < n; i++) {
            cout << ans_id[cv[i]] << " \n"[i == n - 1];
        }
    }

    return true;
}

// Outside subtasks 1-3, n <= 10: try every same-flavour matching of pizzas to people. A stack is
// a run of pizzas whose people come in reverse placement order, so K is the matching's longest
// run increasing in person order: patience piles over the matched people, pruned once the pile
// count reaches the best K so far. At most 10! leaves.
bool solve_small(const vector<int>& arra, const vector<int>& arrb) {
    int n = sz(arra);
    if (n > 10) {
        return false;
    }
    vector<vector<int>> people(n);
    for (int q = n - 1; q >= 0; q--) {
        people[arrb[q]].push_back(q);
    }
    vector<bool> used(n);
    vector<int> tops, pile(n), match(n), best_pile, best_match;
    int best = n + 1;
    auto go = [&](auto& self, int p) -> void {
        if (sz(tops) >= best) {
            return;
        }
        if (p == n) {
            best = sz(tops);
            best_pile = pile;
            best_match = match;
            return;
        }
        for (int q : people[arra[p]]) {
            if (used[q]) {
                continue;
            }
            int i = int(upper_bound(begin(tops), end(tops), q) - begin(tops));
            bool grew = i == sz(tops);
            int old = grew ? -1 : tops[i];
            if (grew) {
                tops.push_back(q);
            } else {
                tops[i] = q;
            }
            used[q] = true;
            pile[p] = i;
            match[p] = q;
            self(self, p + 1);
            used[q] = false;
            if (grew) {
                tops.pop_back();
            } else {
                tops[i] = old;
            }
        }
    };
    go(go, 0);
    cout << best << endl;
    for (int i = 0; i < n; i++) {
        cout << best_pile[i] + 1 << " \n"[i == n - 1];
    }
    vector<int> person_pile(n);
    for (int p = 0; p < n; p++) {
        person_pile[best_match[p]] = best_pile[p];
    }
    for (int q = 0; q < n; q++) {
        cout << person_pile[q] + 1 << " \n"[q == n - 1];
    }
    return true;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arra(n), arrb(n);
    for (auto& a : arra) {
        cin >> a;
        a--;
    }
    for (auto& a : arrb) {
        cin >> a;
        a--;
    }

    if (tally(arra) != tally(arrb)) {
        cout << -1 << endl;
        return;
    }

    bool solved = solve12(arra, arrb) || solve_dist(arra, arrb) || solve_small(arra, arrb);
}

int main() {
    init_io();
    solve();
}