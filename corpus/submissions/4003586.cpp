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

const int maxn = 5e5, logn = 20;
const long mod = 998244353;

int t, tin[maxn], tout[maxn], lift[logn][maxn];
vector<int> graph[maxn];

long bpow(long base, long exp) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp /= 2;
	}
	return ans;
}

long inv(long x) {
	return bpow(x, mod - 2);
}

long deter(vector<vector<long>> arr) {
	int n = sz(arr);
	long ans = 1;
	for (int i = 0; i < n; i++) {
		if (!arr[i][i]) {
			for (int j = i + 1; j < n; j++) {
				if (arr[i][j]) {
					swap(arr[i], arr[j]);
					ans *= -1;
					goto loop;
				}
				j++;
			}
			return 0;
		}
	loop:;
		long cinv = inv(arr[i][i]);
		for (int j = i + 1; j < n; j++) {
			long cmul = (arr[j][i] * cinv) % mod;
			for (int k = i + 1; k < n; k++) {
				arr[j][k] = (arr[j][k] - arr[i][k] * cmul) % mod;
			}
		}
		ans = (ans * arr[i][i]) % mod;
	}
	if (ans < 0) {
		ans += mod;
	}
	return ans;
}

void dfs(int u, int p = -1) {
	tin[u] = t++;
	lift[0][u] = p;
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u);
		}
	}
	tout[u] = t - 1;
}

bool anc(int u, int v) {
	return tin[u] <= tin[v] && tout[v] <= tout[u];
}

int lca(int u, int v) {
	if (anc(u, v)) {
		return u;
	}
	for (int i = logn - 1; i >= 0; i--) {
		if (lift[i][u] != -1 && !anc(lift[i][u], v)) {
			u = lift[i][u];
		}
	}
	return lift[0][u];
}

void solve() {
	int n, k;
	IO::read_int(n, k);
	if (k > n) {
		IO::write_int(0, '\n');
		return;
	}
	int arr[n];
	for (auto& a : arr) {
		IO::read_int(a);
	}
	int lcarr[k];
	for (auto& a : lcarr) {
		IO::read_int(a);
		a--;
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		IO::read_int(u, v);
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	dfs(0);
	for (int i = 1; i < logn; i++) {
		for (int j = 0; j < n; j++) {
			if (lift[i - 1][j] != -1) {
				lift[i][j] = lift[i - 1][lift[i - 1][j]];
			}
		}
	}
	vector<vector<long>> mat(k, vector<long>(k));
	for (int i = 0; i < k; i++) {
		for (int j = 0; j < k; j++) {
			mat[i][j] = arr[lca(lcarr[i], lcarr[j])];
		}
	}
	IO::write_int(deter(mat), '\n');
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