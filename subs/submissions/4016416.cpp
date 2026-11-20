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

namespace IO {
const int BUFFER_SIZE = 1 << 16;

char input_buffer[BUFFER_SIZE];
size_t input_pos = 0, input_len = 0;

char output_buffer[BUFFER_SIZE];
int output_pos = 0;

char number_buffer[100];
uint8_t lookup[100];

void _update_input_buffer() {
	input_len = fread(input_buffer, sizeof(char), BUFFER_SIZE, stdin);
	input_pos = 0;

	if (input_len == 0)
		input_buffer[0] = EOF;
}

inline char next_char(bool advance = true) {
	if (input_pos >= input_len)
		_update_input_buffer();

	return input_buffer[advance ? input_pos++ : input_pos];
}

template <typename T>
inline void read_int(T& number) {
	bool negative = false;
	number = 0;

	while (!isdigit(next_char(false)))
		if (next_char() == '-')
			negative = true;

	do {
		number = 10 * number + (next_char() - '0');
	} while (isdigit(next_char(false)));

	if (negative)
		number = -number;
}

template <typename T, typename... Args>
inline void read_int(T& number, Args&... args) {
	read_int(number);
	read_int(args...);
}

void _flush_output() {
	fwrite(output_buffer, sizeof(char), output_pos, stdout);
	output_pos = 0;
}

inline void write_char(char c) {
	if (output_pos == BUFFER_SIZE)
		_flush_output();

	output_buffer[output_pos++] = c;
}

template <typename T>
inline void write_int(T number, char after = '\0') {
	if (number < 0) {
		write_char('-');
		number = -number;
	}

	int length = 0;

	while (number >= 10) {
		uint8_t lookup_value = lookup[number % 100];
		number /= 100;
		number_buffer[length++] = char((lookup_value & 15) + '0');
		number_buffer[length++] = char((lookup_value >> 4) + '0');
	}

	if (number != 0 || length == 0)
		write_char(char(number + '0'));

	for (int i = length - 1; i >= 0; i--)
		write_char(number_buffer[i]);

	if (after)
		write_char(after);
}

void init() {
	// Make sure _flush_output() is called at the end of the program.
	bool exit_success = atexit(_flush_output) == 0;
	assert(exit_success);

	for (int i = 0; i < 100; i++)
		lookup[i] = uint8_t((i / 10 << 4) + i % 10);
}
}  // namespace IO

const int maxn = 2e6;

struct TSAT {
	int n {}, cind = 1, comp[maxn];
	vector<int> order, graph[2][maxn];
	bool vis[maxn] {}, ans[maxn] {};

	int new_node() { return n++; }

	void add_edge(int u, int v) {
		graph[0][u].push_back(v);
		graph[1][v].push_back(u);
	}

	void either(int a, int b) {
		if (a < 0) {
			a = 2 * (~a) + 1;
		} else {
			a *= 2;
		}
		if (b < 0) {
			b = 2 * (~b) + 1;
		} else {
			b *= 2;
		}
		add_edge(a ^ 1, b);
		add_edge(b ^ 1, a);
	}

	void dfs1(int u) {
		if (vis[u]) {
			return;
		}
		vis[u] = true;
		for (auto& v : graph[0][u]) {
			dfs1(v);
		}
		order.push_back(u);
	}

	void dfs2(int u) {
		if (comp[u]) {
			return;
		}
		comp[u] = cind;
		for (auto& v : graph[1][u]) {
			dfs2(v);
		}
	}

	bool solve() {
		int n = this->n * 2;
		for (int i = 0; i < n; i++) {
			dfs1(i);
		}
		reverse(begin(order), end(order));
		for (auto& a : order) {
			dfs2(a);
			cind++;
		}
		for (int i = 0; i < n / 2; i++) {
			int ti = i * 2, fi = i * 2 + 1;
			if (comp[fi] == comp[ti]) {
				return false;
			} else if (comp[fi] < comp[ti]) {
				ans[i] = true;
			} else {
				ans[i] = false;
			}
		}
		return true;
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	TSAT solver;
	map<int, int> constraints[n];
	auto node_base = [&](int i, int x) -> int {
		auto [it, inserted] = constraints[i].try_emplace(x, 0);
		if (inserted) {
			it->second = solver.new_node();
			solver.new_node();
		}
		return it->second;
	};
	auto lt = [&](int nb) -> int { return nb; };
	auto gt = [&](int nb) -> int { return nb + 1; };
	for (int _ = 0; _ < m; _++) {
		int i, j, k, x;
		cin >> i >> j >> k >> x;
		i--;
		j--;
		k--;
		int inb = node_base(i, x);
		int jnb = node_base(j, x);
		int knb = node_base(k, x);
		solver.either(~lt(inb), ~lt(jnb));
		solver.either(~lt(inb), ~lt(knb));
		solver.either(~lt(jnb), ~lt(knb));
		solver.either(~gt(inb), ~gt(jnb));
		solver.either(~gt(inb), ~gt(knb));
		solver.either(~gt(jnb), ~gt(knb));
	}
	vector<pair<int, int>> nconst[n];
	for (int i = 0; i < n; i++) {
		nconst[i] =
			vector<pair<int, int>>(begin(constraints[i]), end(constraints[i]));
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < sz(nconst[i]); j++) {
			int inb = nconst[i][j].second;
			solver.either(~lt(inb), ~gt(inb));
			if (j < sz(nconst[i]) - 1) {
				int nnb = nconst[i][j + 1].second;
				solver.either(~gt(inb), ~lt(nnb));
				solver.either(gt(inb), lt(nnb));
			}
		}
	}
	if (solver.solve()) {
		IO::write_char('Y');
		IO::write_char('E');
		IO::write_char('S');
		IO::write_char('\n');
		for (int i = 0; i < n; i++) {
			int l = 1, r = 1e9;
			for (auto& [x, nb] : nconst[i]) {
				if (solver.ans[lt(nb)]) {
					r = min(r, x - 1);
				} else {
					l = max(l, x);
				}
				if (solver.ans[gt(nb)]) {
					l = max(l, x + 1);
				} else {
					r = min(r, x);
				}
			}
			IO::write_int(l, i == n - 1 ? '\n' : ' ');
		}
		cout << endl;
	} else {
		IO::write_char('N');
		IO::write_char('O');
		IO::write_char('\n');
	}
}

int main() {
	IO::init();
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}