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

enum Type {
	// person below me is x
	Below,
	// person above me is x
	Above,
	Other
};

struct A {
	string poster, msg;
	Type type;
	int ind;
};

void solve(const vector<A> &arr) {
	int n = sz(arr);
	int lleft[n], lright[n];
	memset(lleft, -1, sizeof(lleft));
	memset(lright, -1, sizeof(lright));
	auto ca = [&](int a, int b) -> bool {
		return a != b && lleft[b] == -1 && lright[a] == -1;
	};
	auto after = [&](int a, int b) -> void {
		assert(ca(a, b));
		lleft[b] = a;
		lright[a] = b;
	};
	{
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				if (ca(i, j) && arr[i].type == Below && arr[j].type == Above &&
					arr[i].poster == arr[j].msg && arr[i].msg == arr[j].poster) {
					after(i, j);
				}
			}
		}
	}
	{
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				if (ca(i, j) && arr[i].type == Below && arr[i].msg == arr[j].poster) {
					after(i, j);
				}
				if (ca(j, i) && arr[i].type == Above && arr[i].msg == arr[j].poster) {
					after(j, i);
				}
			}
		}
	}
	bool vis[n] {};
	vector<int> ans;
	for (int i = 0; i < n; i++) {
		if (vis[i]) {
			continue;
		}
		int u = i;
		while (lleft[u] != -1) {
			u = lleft[u];
		}
		while (u != -1) {
			ans.push_back(u);
			vis[u] = true;
			u = lright[u];
		}
	}
	int cans = 0;
	for (int i = 0; i < n - 1; i++) {
		if (arr[ans[i]].type == Below && arr[ans[i]].msg == arr[ans[i] + 1].poster) {
			cans++;
		}
	}
	for (int i = 1; i < n; i++) {
		if (arr[ans[i]].type == Above && arr[ans[i]].msg == arr[ans[i] - 1].poster) {
			cans++;
		}
	}
	cout << cans << endl;
	for (int i = 0; i < n; i++) {
		cout << ans[i] + 1 << " \n"[i == n - 1];
	}
}

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
	}
	vector<A> arr(m);
	for (int i = 0; i < m; i++) {
		string a, b, c;
		cin >> a >> b >> c;
		Type type = Other;
		if (c == "loushang") {
			type = Below;
		} else if (c == "louxia") {
			type = Above;
		}
		arr[i] = {a, b, type, i};
	}
	solve(arr);
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int tcs;
	cin >> tcs;
	while (tcs--) {
		solve();
	}
}