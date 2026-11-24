#include <bits/stdc++.h>
// A dense 26-letter goto table reserved for every letter read, so only the states made are touched.
int main() {
    int n, m;
    if (scanf("%d %d", &n, &m) != 2) return 1;
    std::vector<std::string> words(n);
    static char buf[1 << 10];
    size_t total = 1;
    for (auto& w : words) {
        if (scanf("%1000s", buf) != 1) return 1;
        w = buf;
        total += w.size();
    }
    std::vector<std::array<int, 26>> nxt;
    nxt.reserve(total);
    nxt.push_back({});
    std::vector<char> end(1);
    for (auto& w : words) {
        int at = 0;
        for (char c : w) {
            int& t = nxt[at][c - 'a'];
            if (!t) {
                t = nxt.size();
                nxt.push_back({});
                end.push_back(0);
            }
            at = nxt[at][c - 'a'];
        }
        end[at] = 1;
    }
    std::vector<int> fail(nxt.size()), q{0};
    for (size_t h = 0; h < q.size(); h++) {
        int s = q[h];
        for (int c = 0; c < 26; c++) {
            int past = s ? nxt[fail[s]][c] : 0;
            int& t = nxt[s][c];
            if (t) {
                fail[t] = past;
                end[t] |= end[past];
                q.push_back(t);
            } else t = past;
        }
    }
    static char text[1 << 20];
    for (int i = 0; i < m; i++) {
        if (scanf("%s", text) != 1) return 1;
        int s = 0, hits = 0;
        for (char* p = text; *p; p++) {
            s = nxt[s][*p - 'a'];
            hits += end[s];
        }
        printf("%d\n", hits);
    }
}
