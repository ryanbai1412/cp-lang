"""One-off migration of the old site's guides into Fumadocs MDX under content/docs/guides."""
import ast
import json
import re
from pathlib import Path

web = Path(__file__).resolve().parent.parent
site = web.parent / "site"
tree = ast.parse((site / "build.py").read_text())
node = next(
    n.value for n in tree.body if isinstance(n, ast.Assign) and getattr(n.targets[0], "id", "") == "REFERENCE_PAGES"
)
pages = [tuple(e.value if isinstance(e, ast.Constant) else "" for e in t.elts) for t in node.elts]
out = web / "content/docs/guides"
out.mkdir(parents=True, exist_ok=True)
order = []
for name, title, section, description, _ in pages:
    src = site / "content" / f"{name}.md"
    if name == "index" or not src.exists():
        continue
    text = src.read_text()
    text = re.sub(r"^```cp(?:[ \t][^\n]*)?$", "```cp", text, flags=re.M)
    text = re.sub(
        r"^```example (\w+)\n```",
        lambda m: "```cp\n" + (site / "examples" / f"{m.group(1)}.cp").read_text().rstrip() + "\n```",
        text,
        flags=re.M,
    )
    text = re.sub(r"\]\(([\w-]+)\.html(#[\w-]+)?\)", lambda m: f"](/docs/guides/{m.group(1)}{m.group(2) or ''})", text)
    text = text.replace("](playground.html)", "](/play)")
    text = re.sub(r"(?<!`)<(?![a-z/!])", "&lt;", text)
    text = text.replace("{", "\\{").replace("}", "\\}")
    text = re.sub(r"(```[\s\S]*?```|`[^`\n]*`)", lambda m: m.group(0).replace("\\{", "{").replace("\\}", "}").replace("&lt;", "<"), text)
    front = f"---\ntitle: {json.dumps(title)}\ndescription: {json.dumps(description)}\n---\n\n"
    (out / f"{name}.mdx").write_text(front + text)
    order.append(name)
(out / "meta.json").write_text(json.dumps({"title": "Guides", "pages": order}, indent=2) + "\n")
print(f"imported {len(order)} guides: {', '.join(order)}")
