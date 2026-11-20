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

#define int long

struct DS {
	vector<long> arr;

	void erase(int x) {
		arr.erase(find(begin(arr), end(arr), x));
	}

	void insert(int x) {
		arr.push_back(x);
	}

	int query(long x) {
		sort(begin(arr), end(arr), greater<>());
		long cur = 0;
		for (int i = 0; i < sz(arr); i++) {
			if (cur >= x) {
				return i;
			}
			cur += arr[i];
		}
		if (cur >= x) {
			return sz(arr);
		}
		return -1;
	}
};

void solve() {
	int n;
	cin >> n;
	pair<int, int> arr[n];
	for (auto& [x, y] : arr) {
		cin >> x >> y;
	}
	int q;
	cin >> q;
	DS ds;
	multiset<pair<int, int>> s;
	for (auto& [x, y] : arr) {
		ds.insert(x - y);
		s.emplace(x, y);
	}
	while (q--) {
		int t;
		cin >> t;
		if (t == 0) {
			int x;
			cin >> x;
			auto [cx, cy] = *(--s.end());
			ds.erase(cx - cy);
			int ans = ds.query(x - cx);
			ds.insert(cx - cy);
			if (ans == -1) {
				cout << -1 << endl;
			} else {
				cout << ans + 1 << endl;
			}
		} else {
			int x, y;
			cin >> x >> y;
			t--;
			s.erase(s.find(arr[t]));
			ds.erase(arr[t].first - arr[t].second);
			arr[t] = {x, y};
			s.insert(arr[t]);
			ds.insert(x - y);
		}
	}
}

int32_t main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}