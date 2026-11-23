package dev.cplang.ide

import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.newvfs.impl.VfsRootAccess
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import org.junit.Assume
import java.io.File
import java.nio.file.Files

/**
 * `cpl lsp` through LSP4IJ, the client every IDE runs it with after `cpl setup jetbrains`
 * (the free IntelliJ IDEA has no other): a wrong file gets the server's diagnostic and
 * completion offers its names.
 */
class CpLsp4ijTest : BasePlatformTestCase() {
    private val cpl = File(System.getProperty("cpl.bin"))

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
        assertTrue("LSP4IJ is loaded", CpLsp4ijFactory.loaded)
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
}
