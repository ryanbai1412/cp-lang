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

void solve() {
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int rarr[n];
    for (int i = 0; i < n; i++) {
        rarr[arr[i]] = i;
    }
    int tout[n];
    {
        set<int> s;
        for (int i = 0, cmn = 0; i < n; i++) {
            s.insert(arr[i]);
            while (sz(s) && *s.begin() == cmn) {
                tout[*s.begin()] = i;
                s.erase(s.begin());
                cmn++;
            }
        }
    }
    vector<int> st[2];
    auto last = [&](int ind) -> int {
        return sz(st[ind]) ? st[ind].back() : 1e9;
    };
    vector<char> ans;
    auto push = [&](int ind, int x) -> void {
        ans.push_back("ac"[ind]);
        st[ind].push_back(x);
    };
    auto pop = [&](int ind) -> void {
        ans.push_back("bd"[ind]);
        st[ind].pop_back();
    };
    int cmn = 0;
    auto process = [&]() -> void {
        while (true) {
            bool changed = false;
            if (last(0) == cmn) {
                pop(0);
                cmn++;
                changed = true;
            }
            if (last(1) == cmn) {
                pop(1);
                cmn++;
                changed = true;
            }
            if (!changed) {
                break;
            }
        }
    };
    int must[n];
    memset(must, -1, sizeof(must));
    for (int i = 0; i < n; i++) {
        process();
        for (int j = i; j < n; j++) {
            for (int k = j + 1; k < n; k++) {
                if (arr[k] > arr[j] && must[arr[k]] != -1 && tout[arr[j]] > k) {
                    must[arr[j]] = must[arr[k]] ^ 1;
                }
            }
        }
        vector<int> lt, gt;
        for (int j = i + 1; j <= tout[arr[i]]; j++) {
            if (arr[j] < arr[i]) {
                lt.push_back(arr[j]);
            } else {
                gt.push_back(arr[j]);
            }
        }
        if (!sz(gt)) {
            if (arr[i] < last(0)) {
                push(0, arr[i]);
            } else if (arr[i] < last(1)) {
                push(1, arr[i]);
            } else {
                dbg(i);
                cout << 0 << endl;
                return;
            }
            continue;
        }
        if (!is_sorted(begin(gt), end(gt), greater<>())) {
            dbg(i);
            cout << 0 << endl;
            return;
        }
        auto check = [&](int ind) -> bool {
            if (must[arr[i]] == (ind ^ 1)) {
                return false;
            }
            if (arr[i] > last(ind) ||
                (gt[0] > last(ind ^ 1) && tout[last(ind ^ 1)] > rarr[gt[0]])) {
                return false;
            }
            for (int j = i + 1; j <= tout[arr[i]]; j++) {
                if (arr[j] > arr[i] && must[arr[j]] == ind) {
                    dbg(j);
                    return false;
                }
            }
            return true;
        };
        int cind;
        if (check(0)) {
            cind = 0;
        } else if (check(1)) {
            cind = 1;
        } else {
            dbg(i, check(0));
            cout << 0 << endl;
            return;
        }
        push(cind, arr[i]);
        for (int j = i + 1; j <= tout[arr[i]]; j++) {
            if (arr[j] > arr[i]) {
                must[arr[j]] = cind ^ 1;
            } else if (arr[j] < arr[i]) {
                if (j < rarr[gt[0]] && tout[arr[j]] > rarr[gt[0]]) {
                    must[arr[j]] = cind;
                }
            }
        }
    }
    process();
    for (int i = 0; i < sz(ans); i++) {
        cout << ans[i] << " \n"[i == sz(ans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}