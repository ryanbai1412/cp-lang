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
#define sz(x) int((x).size())

vector<int> mul(const vector<int> &a, const vector<int> &b) {
	vector<int> ans(sz(a));
	for (int i = 0; i < sz(a); i++) {
		ans[i] = b[a[i]];
	}
	return ans;
}

vector<int> blift(vector<int> base, long exp) {
	vector<int> ans(sz(base));
	iota(begin(ans), end(ans), 0);
	while (exp) {
		if (exp & 1) {
			ans = mul(ans, base);
		}
		exp >>= 1;
		base = mul(base, base);
	}
	return ans;
}

void solve() {
	int n;
	long k;
	cin >> n >> k;
	vector<int> arr(n);
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	arr = blift(arr, k + 1);
	for (int i = 0; i < n; i++) {
		cout << arr[i] + 1 << " \n"[i == n - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}