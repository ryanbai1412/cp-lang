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
#define sz(x) int(size(x))

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

const int bases[] = {2, 2, 3, 3, 3, 3, 3};

vector<vector<int>> devise_strategy(int n) {
    vector<vector<int>> digits;
    for (int i : {-2, 0, 0, -1}) {
        digits.push_back({i});
    }
    for (int i : bases) {
        vector<vector<int>> nxt;
        auto push_one = [&](int x) -> void {
            vector<int> cur {x};
            cur.resize(sz(digits[0]) + 1, x);
            nxt.push_back(cur);
        };
        push_one(-2);
        for (int j = 0; j < i; j++) {
            auto cur = digits;
            for (auto& a : cur) {
                a.insert(a.begin(), j);
            }
            nxt.insert(nxt.end(), begin(cur), end(cur));
        }
        push_one(-1);
        digits = nxt;
        dbg(sz(digits[0]));
    }
    dbg(sz(digits));
    vector<vector<int>> ans;
    int bag = 0;
    auto go = [&](int base, int i) -> void {
        int offset = sz(ans) + base;
        int me = -1 - bag, other = -3 - me;
        vector<int> cans;
        cans.push_back(bag);
        for (auto& a : digits) {
            int cur = a[i];
            if (cur >= 0) {
                cans.push_back(offset + cur);
            } else {
                if (cur == -2) {
                    cans.push_back(me);
                } else {
                    cans.push_back(other);
                }
            }
        }
        for (int j = 0; j < base; j++) {
            auto xans = cans;
            if (i) {
                for (int k = 0; k < sz(digits); k++) {
                    if (digits[k][i - 1] >= 0 && digits[k][i - 1] != j) {
                        if (digits[k][i - 1] < j) {
                            xans[k + 1] = me;
                        } else {
                            xans[k + 1] = other;
                        }
                    }
                }
            }
            ans.push_back(xans);
        }
        bag ^= 1;
    };
    go(1, 0);
    for (int i = 0; i < sz(bases); i++) {
        go(bases[sz(bases) - i - 1], i + 1);
    }
    dbg(sz(digits[0]), sz(bases));
    {
        int me = -1 - bag, other = -3 - me;
        vector<int> cans;
        cans.push_back(bag);
        int used = 0;
        for (int j = 0; j < sz(digits); j++) {
            if (digits[j].back() >= 0) {
                if (used % 2 == 0) {
                    cans.push_back(me);
                } else {
                    cans.push_back(other);
                }
                used++;
            } else {
                if (digits[j].back() == -2) {
                    cans.push_back(me);
                } else {
                    cans.push_back(other);
                }
            }
        }
        ans.push_back(cans);
    }
    for (auto& a : ans) {
        a.resize(n + 1);
    }
    dbg(digits[7], digits[8]);
    dbg(sz(ans));
    //    for (auto& a : ans) {
    //        for (auto& b : a) {
    //            cerr << b << " ";
    //        }
    //        cerr << endl;
    //    }
    return ans;
}