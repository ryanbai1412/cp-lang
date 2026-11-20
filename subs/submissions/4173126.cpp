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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
	tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	double ans = 0;
	ost<pair<int, int>> s;
	for (int i = 0; i < n; i++) {
		s.insert({arr[i], i});
		ans += sz(s) - s.order_of_key({arr[i], 1e9});
	}
	cout << 1 + (ans / n) << endl;
}

int main() {
	cout << fixed << setprecision(2);
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}