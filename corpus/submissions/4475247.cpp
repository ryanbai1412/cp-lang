#include <ostream>
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

const int maxn = 505;

struct Sol {
	int w, meq;
	array<int, 6> ans;
	vector<int> neq;

	bool operator<(const Sol& s) const {
		return w < s.w;
	}
};

int n, m, arr[maxn][6];

int opt(int ind, const vector<int>& ign) {
	bool bad[n] {};
	for (auto& a : ign) {
		bad[a] = true;
	}
	pair<int, int> ans {-1, -1};
	for (int i = 0; i < n; i++) {
		if (!bad[i]) {
			ans = max(ans, pair<int, int> {arr[i][ind], i});
		}
	}
	return ans.second;
}

vector<Sol> fracture(const Sol& s) {
	vector<Sol> ans;
	vector<int> ineq = s.neq;
	for (int i = s.meq; i < m; i++) {
		ineq.push_back(s.ans[i]);
		auto cneq = ineq;
		int w[6] {};
		array<int, 6> cur {};
		copy(s.ans.begin(), s.ans.begin() + i, cur.begin());
		for (int j = i; j < m; j++) {
			cur[j] = opt(j, cneq);
			if (cur[j] == -1) {
				goto loop;
			}
			cneq.push_back(cur[j]);
		}
		for (int j = 0; j < m; j++) {
			for (int k = 0; k < m; k++) {
				w[k] = max(w[k], arr[cur[j]][k]);
			}
		}
		ineq.push_back(cur[i]);
		ans.push_back(Sol {
			accumulate(begin(w), end(w), 0),
			i,
			cur,
			ineq
		});
		ineq.pop_back();
	loop:;
	}
	return ans;
}

Sol iopt() {
	int w[6] {};
	array<int, 6> cur {};
	vector<int> neq;
	for (int i = 0; i < m; i++) {
		cur[i] = opt(i, neq);
		neq.push_back(cur[i]);
	}
	for (int i = 0; i < m; i++) {
		for (int j = 0; j < m; j++) {
			w[j] = max(w[j], arr[cur[i]][j]);
		}
	}
	return {accumulate(begin(w), end(w), 0), 0, cur, {}};
}

void solve() {
	int k;
	cin >> n >> m >> k;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			cin >> arr[i][j];
		}
	}
	dbg(sz(fracture({0, 0, {1, 0, 0, 0, 0, 0}, {1, 2}})));
	priority_queue<Sol> pq;
	pq.push(iopt());
	for (int i = 0; i < k - 1; i++) {
		auto x = pq.top();
		dbg(x.ans[0], x.ans[1], sz(x.neq), sz(fracture(x)));
		pq.pop();
		for (auto& a : fracture(x)) {
			pq.push(a);
			dbg(a.ans[0], a.ans[1]);
		}
		dbg("D");
	}
	dbg(sz(pq));
	cout << pq.top().w << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}