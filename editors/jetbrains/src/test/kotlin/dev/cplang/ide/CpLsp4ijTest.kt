package dev.cplang.ide

import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.actionSystem.ActionManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.newvfs.impl.VfsRootAccess
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.intellij.openapi.util.Disposer
import org.junit.Assume
import java.io.File
import java.net.Socket
import java.nio.file.Files

/**
 * `cpl lsp` through LSP4IJ, the client every IDE runs it with after `cpl setup jetbrains`
 * (the free IntelliJ IDEA has no other): a wrong file gets the server's diagnostic, a
 * lint shows as a warning whose quick fix rewrites the file, and completion offers its names.
 */
class CpLsp4ijTest : BasePlatformTestCase() {
    private val cpl = File(System.getProperty("cpl.bin"))
    private val repo = File(System.getProperty("cpl.repo"))

    override fun setUp() {
        super.setUp()
        Assume.assumeTrue("build cpl first: cargo build -p cpl-cli", cpl.canExecute())
        CplSettings.get().state.path = cpl.path
    }

    private fun open(name: String, text: String) {
        val dir = Files.createTempDirectory("cpl").toFile()
        VfsRootAccess.allowRootAccess(testRootDisposable, dir.canonicalPath)
        val file = File(dir, name).apply { writeText(text) }
        myFixture.configureFromExistingVirtualFile(LocalFileSystem.getInstance().refreshAndFindFileByIoFile(file)!!)
    }

    /** Polls `answer` while the server starts and answers, for up to a minute. */
    private fun <T : Any> eventually(what: String, answer: () -> T?): T {
        val until = System.currentTimeMillis() + 60_000
        while (System.currentTimeMillis() < until) {
            PlatformTestUtil.dispatchAllInvocationEventsInIdeEventQueue()
            answer()?.let { return it }
            Thread.sleep(250)
        }
        fail("no $what from `cpl lsp` through LSP4IJ in 60 s")
        error("unreachable")
    }

    fun `test lsp4ij is the client`() {
        val id = com.intellij.openapi.extensions.PluginId.getId("com.redhat.devtools.lsp4ij")
        assertFalse("LSP4IJ is loaded", com.intellij.ide.plugins.PluginManagerCore.isDisabled(id))
        assertNotNull(com.intellij.ide.plugins.PluginManagerCore.getPlugin(id))
    }

    fun `test the server's code lenses run through the cpl exec action`() {
        assertEquals(mapOf("codeLens" to true), CpLsp4ijFactory.INITIALIZATION_OPTIONS)
        assertTrue(com.intellij.openapi.actionSystem.ActionManager.getInstance().getAction("cpl.exec") is CplExecAction)
    }

    /** A lens's `cpl.exec`, run by the server: its answer once the run ends. */
    private fun exec(vararg args: String): com.google.gson.JsonObject {
        val run = CplExecAction.run(project, args.toList())
        return eventually("cpl.exec's answer") { run.takeIf { it.isDone }?.get() }
    }

    fun `test the stress lens saves a failing case that the test lens reruns`() {
        val planted = File(repo, "examples/debugging/knapsack.cp").readText()
        assertTrue(planted.contains("for c in 0..=cap:"))
        open("k.cp", planted)
        val source = File(myFixture.file.virtualFile.path)
        val stressed = exec("stress", source.path)
        assertEquals(stressed.toString(), 1, stressed.get("code").asInt)
        assertEquals(1, File(source.parentFile, "k.tests").listFiles { f -> f.name.endsWith(".in") }.orEmpty().size)
        val failing = exec("test", source.path)
        assertTrue(failing.get("code").asInt != 0)
        val tree = CplTests.show(project, source, failing.getAsJsonObject("view").getAsJsonArray("tests"))
        val cases = tree.resultsViewer.testsRootNode.allTests.filter { it.isLeaf }
        assertTrue(cases.toString(), cases.isNotEmpty() && cases.any { it.isDefect })
        Disposer.dispose(tree)
        source.writeText(planted.replace("for c in 0..=cap:", "for c in (0..=cap).rev:"))
        assertEquals(0, exec("test", source.path).get("code").asInt)
    }

    fun `test a wrong call is the server's diagnostic on its line`() {
        open("p.cp", File(repo, "crates/cpl-cli/tests/debugger/part_planted.cp").readText())
        val source = File(myFixture.file.virtualFile.path)
        val one = File(source.parentFile, "one.in").apply { writeText("1\n") }
        exec("locate", source.path, one.path)
        val marked = eventually("wrong call") {
            myFixture.doHighlighting(HighlightSeverity.ERROR).firstOrNull { it.description?.startsWith("first wrong call") == true }
        }
        assertEquals(3, myFixture.editor.document.getLineNumber(marked.startOffset))
    }

    fun `test a trace lens answers the page the trace window shows`() {
        open("t.cp", "fn main():\n    print(1)\n")
        val traced = exec("run", File(myFixture.file.virtualFile.path).path, "--trace")
        assertTrue(traced.get("html").asString.contains("<html", ignoreCase = true))
    }

    fun `test import a problem opens the file the server creates`() {
        open("seed.cp", "fn main():\n    print(1)\n")
        myFixture.testAction(ActionManager.getInstance().getAction("CpLang.ImportProblem"))
        val problem = """{"name":"D. Range Max","group":"Codeforces","url":"https://codeforces.com/problemset/problem/1/D",""" +
            """"tests":[{"input":"3\n","output":"6\n"}],"timeLimit":1500,"memoryLimit":512,"batch":{"id":"b","size":1}}"""
        eventually("Competitive Companion's port") {
            runCatching {
                Socket("127.0.0.1", 10043).use { socket ->
                    val body = problem.toByteArray()
                    socket.getOutputStream().write("POST / HTTP/1.1\r\nContent-Length: ${body.size}\r\n\r\n".toByteArray() + body)
                    socket.getInputStream().readBytes()
                }
            }.getOrNull()
        }
        val opened = eventually("opened problem") {
            FileEditorManager.getInstance(project).openFiles.firstOrNull { it.extension == "cp" && it.name != "seed.cp" }
        }
        assertTrue(String(opened.contentsToByteArray()).contains("Range Max"))
    }

    fun `test a wrong file gets the server's diagnostic`() {
        open("bad.cp", "fn main():\n    let y: Int = \"text\"\n    print(y)\n")
        val error = eventually("diagnostic") {
            myFixture.doHighlighting(HighlightSeverity.ERROR).firstOrNull { it.description != null }
        }
        assertEquals(1, myFixture.editor.document.getLineNumber(error.startOffset))
    }

    fun `test completion offers the server's names`() {
        open("done.cp", "fn main():\n    let total = 3\n    let totem = 4\n    print(tot<caret> + totem)\n")
        eventually("completion") {
            // LSP4IJ's items, not a word the IDE found in the file itself.
            myFixture.completeBasic()?.filter { item ->
                listOf(item, item.`object`).any { it.javaClass.name.startsWith("com.redhat.devtools.lsp4ij") }
            }?.takeIf { items -> items.any { it.lookupString == "total" } }
        }
    }

    fun `test a lint is a warning whose quick fix applies`() {
        open("lint.cp", "let x = 3\nif x >= 0 and x < 5: print(x)\n")
        val warning = eventually("lint warning") {
            myFixture.doHighlighting(HighlightSeverity.WARNING)
                .firstOrNull { it.severity == HighlightSeverity.WARNING && it.description?.contains("range check") == true }
        }
        assertEquals(1, myFixture.editor.document.getLineNumber(warning.startOffset))
        myFixture.editor.caretModel.moveToOffset(warning.startOffset)
        val fix = eventually("quick fix") {
            myFixture.doHighlighting(HighlightSeverity.WARNING)
            myFixture.availableIntentions.firstOrNull { it.text.contains("0 <= x < 5") }
        }
        myFixture.launchAction(fix)
        eventually("fixed text") {
            PlatformTestUtil.dispatchAllInvocationEventsInIdeEventQueue()
            myFixture.editor.document.text.takeIf { it.contains("if 0 <= x < 5:") }
        }
    }
}
