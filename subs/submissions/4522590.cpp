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

void solve() {
	int n, m;
	cin >> n >> m;
	multiset<int> arr;
	for (int i = 0; i < n; i++) {
		int x;
		cin >> x;
		arr.insert(x);
	}
	vector<int> ans;
	while (sz(arr)) {
		int x = *arr.begin();
		ans.push_back(x);
		arr.erase(arr.begin());
		if (!sz(arr)) {
			break;
		}
		auto it = arr.lower_bound(m - x);
		if (it == arr.end()) {
			cout << -1 << endl;
			return;
		}
		ans.push_back(*it);
		arr.erase(it);
	}
	for (int i = 0; i < n; i++) {
		cout << ans[i] << " \n"[i == n - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}