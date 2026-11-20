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

const int maxn = 5000;

int n, m;
vector<int> own, dep, igraph[maxn];

vector<int> reachable(const vector<int>& arr, int side) {
	vector<int> dep = ::dep;

	bool qd[n]{};
	vector<int> q;
	auto push = [&](int u, bool f = false) -> void {
		if (!qd[u] && (f || !dep[u])) {
			qd[u] = true;
			q.push_back(u);
		}
	};

	for (int i = 0; i < n; i++) {
		if (arr[i]) {
			push(i, true);
		}
	}

	vector<int> ans(n);
	while (sz(q)) {
		int u = q.back();
		q.pop_back();

		ans[u] = true;

		for (auto& v : igraph[u]) {
			dep[v]--;
			if (own[v] == side) {
				push(v, true);
			} else {
				push(v);
			}
		}
	}

	return ans;
}

vector<int> who_wins(vector<int> own,
					 vector<int> charging,
					 vector<int> us,
					 vector<int> vs) {
	::own = own;
	n = sz(own);
	m = sz(us);

	for (int i = 0; i < m; i++) {
		int u = us[i], v = vs[i];
		igraph[v].push_back(u);
	}

	dep.resize(n);
	for (auto& a : us) {
		dep[a]++;
	}

	vector<int> init;
	{
		init = reachable(charging, 1);
		for (auto& a : init) {
			a ^= 1;
		}
	}

	while (true) {
		vector<int> nxt = reachable(init, 0);
		
		if (init == nxt) {
			break;
		}
		init = nxt;
	}

	vector<int> ans = init;
	for (auto& a : ans) {
		a ^= 1;
	}

	return ans;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, m;
	cin >> n >> m;
	vector<int> own(n), charging(n), us(m), vs(m);
	for (auto& a : own) {
		cin >> a;
	}
	for (auto& a : charging) {
		cin >> a;
	}
	for (int i = 0; i < m; i++) {
		cin >> us[i] >> vs[i];
	}
	auto ans = who_wins(own, charging, us, vs);
	assert(sz(ans) == n);
	for (auto& a : ans) {
		cout << a << " ";
	}
	cout << endl;
}