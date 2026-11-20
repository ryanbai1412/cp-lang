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
T reversed(T t) {
    reverse(begin(t), end(t));
    return t;
}

template <typename T>
bool is_palin(const T& t) {
    return t == reversed(t);
}

vector<int> add(const vector<int>& a, const vector<int>& b, int base) {
    assert(sz(a) == sz(b));
    vector<int> ans;
    int carry = 0;
    for (int i = 0; i < sz(a); i++) {
        int cx = a[i] + b[i] + carry;
        carry = cx / base;
        ans.push_back(cx % base);
    }
    if (carry) {
        ans.push_back(carry);
    }
    return reversed(ans);
}

void solve() {
    int base;
    string s;
    cin >> base >> s;
    vector<int> arr;
    for (auto& a : s) {
        if (isdigit(a)) {
            arr.push_back(a - '0');
        } else {
            arr.push_back(10 + a - 'a');
        }
    }
    for (int i = 0; i <= 30; i++) {
        if (is_palin(arr)) {
            cout << "STEP=" << i << endl;
            return;
        }
        arr = add(arr, reversed(arr), base);
    }
    cout << "Impossible!" << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}