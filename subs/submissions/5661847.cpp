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
 
using ll = long long;
 
#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))
 
namespace __gnu_pbds {
template <typename T>
using ost =
    tree<T, null_type, less<T>, rb_tree_tag, tree_order_statistics_node_update>;
}
 
using __gnu_pbds::ost;
 
long count_inversions(const vector<int>& arr) {
    ost<pair<int, int>> ost;
    int n = sz(arr);
    long ans = 0;
    for (int i = 0; i < n; i++) {
        ans += sz(ost) - int(ost.order_of_key({arr[i] + 1, -1}));
        ost.insert({arr[i], i});
    }
    return ans;
}
 
long count1(vector<pair<int, int>> arr) {
    map<int, pair<bool, int>> evts;
    for (auto& [ql, qr] : arr) {
        evts[ql] = {true, qr};
        evts[qr] = {false, qr};
    }
 
    ost<int> ost;
    long ans = 0;
 
    for (auto& [_t, cp] : evts) {
        auto& [b, x] = cp;
        if (b) {
            ost.insert(x);
            ans += int(ost.order_of_key(x));
            ans += 2 * (sz(ost) - int(ost.order_of_key(x)));
        } else {
            ost.erase(x);
        }
    }
 
    return ans;
}
 
long solve1(vector<pair<int, int>> arr) {
    for (auto& [ql, qr] : arr) {
        if (ql > qr) {
            swap(ql, qr);
        }
    }
 
    return count1(arr);
 
    // long ans = 0;
 
    // for (auto& [l1, r1] : arr) {
    //     for (auto& [l2, r2] : arr) {
    //         if (l1 <= l2 && r2 <= r1) {
    //             ans += 2;
    //         } else if (l1 <= l2 && l2 <= r1 && r1 <= r2) {
    //             ans++;
    //         }
    //     }
    // }
 
    // return ans;
}
 
ll count_swaps(vector<int> arr) {
    int n = sz(arr);
 
    map<int, vector<pair<int, int>>> mp;
    for (int i = 0; i < n; i++) {
        mp[abs(arr[i])].emplace_back(arr[i], i);
    }
 
    vector<pair<int, int>> s_pairs;
 
    long ans = 0;
 
    for (auto& [k, v] : mp) {
        vector<int> t_pos;
        for (auto& [_x, i] : v) {
            t_pos.push_back(i);
        }
 
        int neg = 0, pos = 1;
        vector<int> carr;
        int rmap[sz(v)];
 
        for (int i = 0; i < sz(v); i++) {
            auto& [x, ei] = v[i];
            if (x < 0) {
                rmap[neg] = ei;
                carr.push_back(t_pos[neg]);
                neg += 2;
            } else {
                rmap[pos] = ei;
                carr.push_back(t_pos[pos]);
                pos += 2;
            }
        }
        ans += count_inversions(carr);
 
        vector<pair<int, int>> c_s;
        for (int i = 0; i < sz(v); i += 2) {
            c_s.emplace_back(rmap[i], rmap[i + 1]);
        }
 
        ans -= solve1(c_s);
        s_pairs.insert(s_pairs.end(), begin(c_s), end(c_s));
    }
 
    ans += solve1(s_pairs);
 
    return ans;
}