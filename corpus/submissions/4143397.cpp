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

void solve() {
	int n;
	cin >> n;
	vector<int> cur;
	for (int i = 0; i < n; i++) {
		string t, s;
		cin >> t >> s;
		int ind = 1;
		if (t == "subsection") {
			ind = 2;
		} else if (t == "subsubsection") {
			ind = 3;
		}
		bool inc = ind <= sz(cur);
		cur.resize(ind, 1);
		if (inc) {
			cur.back()++;
		}
		for (int j = 0; j < ind; j++) {
			cout << cur[j] << ". "[j == ind - 1];
		}
		cout << s << endl;
	}
}

int main() {
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