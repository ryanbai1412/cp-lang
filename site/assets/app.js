const root = document.documentElement;
const themeButton = document.querySelector(".theme-toggle");
function updateThemeLabel() {
  themeButton.setAttribute(
    "aria-label",
    `Switch to ${root.dataset.theme === "dark" ? "light" : "dark"} theme`,
  );
}
updateThemeLabel();
themeButton.addEventListener("click", () => {
  root.dataset.theme = root.dataset.theme === "dark" ? "light" : "dark";
  try {
    localStorage.setItem("cp-theme", root.dataset.theme);
  } catch (_) {
    /* Storage is optional. */
  }
  updateThemeLabel();
});

const menu = document.querySelector(".menu-toggle");
const sidebar = document.querySelector(".sidebar");
const mobileLayout = matchMedia("(max-width: 760px)");
function updateNavigationVisibility() {
  sidebar.inert =
    mobileLayout.matches && !document.body.classList.contains("menu-open");
}
function closeMenu() {
  if (
    document.body.classList.contains("menu-open") &&
    sidebar.contains(document.activeElement)
  )
    menu.focus();
  document.body.classList.remove("menu-open");
  menu.setAttribute("aria-expanded", "false");
  menu.setAttribute("aria-label", "Open navigation");
  updateNavigationVisibility();
}
menu.addEventListener("click", () => {
  const open = document.body.classList.toggle("menu-open");
  menu.setAttribute("aria-expanded", String(open));
  menu.setAttribute("aria-label", `${open ? "Close" : "Open"} navigation`);
  updateNavigationVisibility();
});
mobileLayout.addEventListener("change", closeMenu);
updateNavigationVisibility();
document.addEventListener("click", (event) => {
  if (!event.target.closest(".sidebar, .menu-toggle")) closeMenu();
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") closeMenu();
});

document.querySelectorAll(".copy-button").forEach((button) => {
  button.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(
        button.closest("figure").querySelector("code").textContent,
      );
      button.textContent = "Copied";
    } catch (_) {
      button.textContent = "Select code to copy";
    }
    setTimeout(() => {
      button.textContent = "Copy";
    }, 2000);
  });
});

const dialog = document.querySelector(".search-dialog");
const input = document.querySelector("#search-input");
const results = document.querySelector("#search-results");
const status = document.querySelector("#search-status");
let indexPromise;
let selected = -1;
let queryVersion = 0;
function openSearch() {
  if (!dialog.open) dialog.showModal();
  input.focus();
}
document.querySelector(".search-trigger").addEventListener("click", openSearch);
document.addEventListener("keydown", (event) => {
  const editing = event.target.closest("input, textarea, [contenteditable]");
  if (
    (event.key === "/" && !editing) ||
    ((event.ctrlKey || event.metaKey) && event.key === "k")
  ) {
    event.preventDefault();
    openSearch();
  }
});
dialog.addEventListener("click", (event) => {
  if (event.target === dialog) dialog.close();
});
input.addEventListener("input", async () => {
  const version = ++queryVersion;
  const query = input.value.trim().toLowerCase();
  results.replaceChildren();
  selected = -1;
  if (!query) {
    status.textContent = "Type to find a chapter or a concept.";
    return;
  }
  status.textContent = "Searching…";
  try {
    indexPromise ||= fetch("search.json")
      .then((response) => {
        if (!response.ok) throw new Error("Search index unavailable");
        return response.json();
      })
      .catch((error) => {
        indexPromise = null;
        throw error;
      });
    const pages = await indexPromise;
    if (version !== queryVersion) return;
    const words = query.split(/\s+/);
    const hits = pages
      .filter((page) =>
        words.every((word) =>
          `${page.title} ${page.text}`.toLowerCase().includes(word),
        ),
      )
      .sort(
        (a, b) =>
          Number(b.title.toLowerCase().includes(query)) -
          Number(a.title.toLowerCase().includes(query)),
      );
    status.textContent = hits.length
      ? `${hits.length} chapter${hits.length === 1 ? "" : "s"} found`
      : "No matches. Try “Index”, “ASCII”, or “memo”.";
    hits.forEach((page) => {
      const link = document.createElement("a");
      link.href = page.url;
      const title = document.createElement("strong");
      title.textContent = page.title;
      const excerpt = document.createElement("span");
      const at = Math.max(0, page.text.toLowerCase().indexOf(words[0]) - 45);
      excerpt.textContent =
        (at ? "…" : "") + page.text.slice(at, at + 155) + "…";
      link.append(title, excerpt);
      results.append(link);
    });
  } catch (_) {
    if (version === queryVersion)
      status.textContent =
        "Search could not load. Browse the chapters in the sidebar, or try again.";
  }
});
dialog.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    event.preventDefault();
    dialog.close();
    return;
  }
  const links = [...results.querySelectorAll("a")];
  if (!links.length) return;
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    selected =
      (selected + (event.key === "ArrowDown" ? 1 : -1) + links.length) %
      links.length;
    links.forEach((link, i) =>
      link.classList.toggle("selected", i === selected),
    );
    links[selected].focus();
  } else if (event.key === "Enter" && event.target === input) {
    event.preventDefault();
    links[Math.max(selected, 0)].click();
  }
});

const tocLinks = [...document.querySelectorAll('.toc a[href^="#"]')];
if ("IntersectionObserver" in window) {
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting)
          tocLinks.forEach((link) =>
            link.classList.toggle(
              "active",
              link.hash === `#${entry.target.id}`,
            ),
          );
      }
    },
    { rootMargin: "-10% 0px -65% 0px" },
  );
  document
    .querySelectorAll("article h2")
    .forEach((heading) => observer.observe(heading));
}
