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

vector<int> who_wins(vector<int> own,
					 vector<int> charging,
					 vector<int> us,
					 vector<int> vs) {
	int n = sz(own), m = sz(us);

	vector<int> igrapha[n], igraphb[n];
	for (int i = 0; i < m; i++) {
		if (charging[us[i]]) {
			igraphb[vs[i]].push_back(us[i]);
		} else {
			igrapha[vs[i]].push_back(us[i]);
		}
	}

	int pn = n;
	n = n * (n + 1);

	auto encode = [&](int u, int d) -> int { return d * pn + u; };

	int dep[n]{};
	{
		for (auto& a : us) {
			if (charging[a]) {
				dep[encode(a, 0)]++;
			} else {
				for (int i = 0; i < pn; i++) {
					dep[encode(a, i)]++;
				}
			}
		}
	}

	bool qd[n]{};
	vector<int> q;
	q.reserve(n);

	auto push = [&](int u, bool f = false) -> void {
		if (!qd[u] && (f || !dep[u])) {
			qd[u] = true;
			q.push_back(u);
		}
	};
	for (int i = 0; i < pn; i++) {
		if (charging[i]) {
			push(encode(i, 0));
		} else {
			for (int j = 0; j <= pn; j++) {
				push(encode(i, j));
			}
		}
	}

	vector<int> ans(n, -1);
	while (sz(q)) {
		int cur = q.back();
		q.pop_back();
		int j = cur / pn, u = cur % pn;

		if (ans[cur] == -1) {
			ans[cur] = false;
		}
		dbg(u, j, ans[cur]);

		auto process = [&](int v, int en) -> void {
			dep[en]--;
			if (!own[v]) {
				ans[en] = false;
				push(en, true);
			} else {
				push(en);
			}
		};

		if (charging[u]) {
			assert(j == 0);
			for (auto& v : igrapha[u]) {
				for (int j = 0; j < pn; j++) {
					process(v, encode(v, j));
				}
			}
			for (auto& v : igraphb[u]) {
				process(v, encode(v, 0));
			}
		} else {
			if (j) {
				for (auto& v : igrapha[u]) {
					process(v, encode(v, j - 1));
				}
			}
			if (j == 1) {
				for (auto& v : igraphb[u]) {
					process(v, encode(v, 0));
				}
			}
		}
	}

	for (auto& a : ans) {
		if (a == -1) {
			a = true;
		}
	}

	vector<int> xans(pn);
	for (int i = 0; i < pn; i++) {
		xans[i] = ans[encode(i, 0)];
		
		if (xans[i] == -1) {
			xans[i] = true;
		}
	}

	return xans;
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