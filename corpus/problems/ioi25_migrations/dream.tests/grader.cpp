// The judge's grader, as the sample grader reads and prints: N, then P[1..N-1]; prints
// S[1..N-1], then U V. Like the real grader, send_message and longest_path run in separate
// processes, so the second phase sees none of the first phase's state.
#include <cstdio>
#include <sys/wait.h>
#include <unistd.h>
#include <utility>
#include <vector>

int send_message(int N, int i, int Pi);
std::pair<int, int> longest_path(std::vector<int> S);

int main() {
  int N;
  if (std::scanf("%d", &N) != 1) return 1;
  std::vector<int> P(N, 0), S(N, 0);
  for (int i = 1; i < N; ++i)
    if (std::scanf("%d", &P[i]) != 1) return 1;
  std::fflush(stdout);
  int fd[2];
  if (pipe(fd)) return 1;
  pid_t first = fork();
  if (first < 0) return 1;
  if (first == 0) {
    close(fd[0]);
    for (int i = 1; i < N; ++i) S[i] = send_message(N, i, P[i]);
    if (write(fd[1], S.data(), N * sizeof(int)) != (ssize_t)(N * sizeof(int))) _exit(1);
    _exit(0);
  }
  close(fd[1]);
  size_t got = 0;
  while (got < N * sizeof(int)) {
    ssize_t r = read(fd[0], (char *)S.data() + got, N * sizeof(int) - got);
    if (r <= 0) break;
    got += r;
  }
  int status;
  waitpid(first, &status, 0);
  if (got != N * sizeof(int) || !WIFEXITED(status) || WEXITSTATUS(status)) return 1;
  for (int i = 1; i < N; ++i) std::printf("%d%c", S[i], " \n"[i + 1 == N]);
  auto [U, V] = longest_path(S);
  std::printf("%d %d\n", U, V);
  return 0;
}
