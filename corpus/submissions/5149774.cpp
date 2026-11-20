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

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "{" << p.first << ", " << p.second << "}";
}

vector<int> map_first(const vector<pair<int, int>>& arr) {
    vector<int> ans;
    for (auto& a : arr) {
        ans.push_back(a.first);
    }
    return ans;
}

long min_dist1(vector<int> arr) {
    sort(begin(arr), end(arr));
    long ans = 0;
    for (auto& a : arr) {
        ans += abs(a - arr[sz(arr) / 2]);
    }
    return ans;
}

long solve(vector<pair<int, int>> arr) {
    sort(begin(arr), end(arr));
    long ans = min_dist1(map_first(arr));
    for (int i = 0; i <= sz(arr); i++) {
        long cans = min_dist1(map_first(
                        vector<pair<int, int>>(arr.begin(), arr.begin() + i))) +
                    min_dist1(map_first(
                        vector<pair<int, int>>(arr.begin() + i, arr.end())));
        for (int j = 0; j < sz(arr); j++) {
            cans += arr[j].second == (j < i);
        }
        ans = min(ans, cans);
    }
    dbg(arr, ans);
    return ans;
}

void solve() {
    int n;
    cin >> n;
    int arr[n][3];
    for (auto& [x, y, v] : arr) {
        cin >> x >> y >> v;
    }
    long ans = 0;
    {
        vector<pair<int, int>> vx;
        for (auto& [x, _y, v] : arr) {
            if (v == 0 || v == 1) {
                vx.emplace_back(x, 0);
            } else {
                vx.emplace_back(x, 1);
            }
        }
        ans += solve(vx);
    }
    {
        vector<pair<int, int>> vy;
        for (auto& [_x, y, v] : arr) {
            if (v == 0 || v == 3) {
                vy.emplace_back(y, 0);
            } else {
                vy.emplace_back(y, 1);
            }
        }
        ans += solve(vy);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}