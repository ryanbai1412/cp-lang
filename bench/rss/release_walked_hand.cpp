#include <cstdio>
#include <string>
#include <vector>

int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<std::string> words(n);
    static char buf[64];
    for (auto& w : words) {
        if (scanf("%63s", buf) != 1) return 1;
        w = buf;
    }
    std::vector<long long> keys(n);
    for (int i = 0; i < n; i++) keys[i] = (long long)words[i].size() * 31;
    std::vector<std::string>().swap(words);
    std::vector<long long> big(4LL * n);
    for (long long i = 0; i < 4LL * n; i++) big[i] = keys[i % n] + i;
    printf("%lld\n", big[4LL * n - 1] + big[0]);
}
