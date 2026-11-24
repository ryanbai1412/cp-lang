package dev.cplang.ide

import com.google.gson.Gson
import com.google.gson.JsonObject
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.redhat.devtools.lsp4ij.LanguageServerManager
import org.eclipse.lsp4j.ExecuteCommandParams
import java.util.concurrent.CompletableFuture
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.editor.LogicalPosition
import com.intellij.openapi.editor.ScrollType
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.RangeHighlighter
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.OpenFileDescriptor
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.openapi.wm.ToolWindowManager
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.jcef.JBCefBrowser
import com.intellij.xdebugger.ui.DebuggerColors
import org.cef.browser.CefBrowser
import org.cef.browser.CefFrame
import org.cef.browser.CefMessageRouter
import org.cef.callback.CefQueryCallback
import org.cef.handler.CefMessageRouterHandlerAdapter
import com.redhat.devtools.lsp4ij.LanguageServerFactory
import com.redhat.devtools.lsp4ij.commands.LSPCommand
import com.redhat.devtools.lsp4ij.commands.LSPCommandAction
import com.redhat.devtools.lsp4ij.server.OSProcessStreamConnectionProvider
import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider
import com.intellij.openapi.vfs.VirtualFile
import java.io.File

/** `cpl lsp` through LSP4IJ in every IDE; `cpl setup jetbrains` installs LSP4IJ (decision 617). */
class CpLsp4ijFactory : LanguageServerFactory {
    override fun createConnectionProvider(project: Project): StreamConnectionProvider =
        object : OSProcessStreamConnectionProvider(
            Cpl.commandLine(Cpl.lspArgs(), project.basePath?.let(::File)?.takeIf { it.isDirectory }),
        ) {
            override fun getInitializationOptions(rootUri: VirtualFile?): Any = INITIALIZATION_OPTIONS
        }

    companion object {
        /** Asks `cpl lsp` for its Run, Trace, Test and Stress code lenses. */
        val INITIALIZATION_OPTIONS = mapOf("codeLens" to true)
    }
}

/**
 * `cpl.exec`, the command of `cpl lsp`'s code lenses: the server runs it and shows its report through
 * the protocol (the panel, the status, each line's mark, the wrong call as a diagnostic); a trace's
 * page comes back as `html` for the Trace tool window, and a test run's cases as `view.tests` for the test tree.
 */
class CplExecAction : LSPCommandAction() {
    override fun getCommandPerformedThread() = ActionUpdateThread.EDT

    override fun commandPerformed(command: LSPCommand, e: AnActionEvent) {
        val project = e.getRequiredData(CommonDataKeys.PROJECT)
        FileDocumentManager.getInstance().saveAllDocuments()
        val source = File(command.getArgumentAt(1, String::class.java)!!)
        run(project, command.arguments).thenAccept {
            CplTrace.open(project, source, it)
            CplTests.open(project, source, it)
        }
    }

    companion object {
        /** `cpl.exec` of a lens's arguments in `cpl lsp`: its `{view, html, code, stderr}` once the run ends. */
        fun run(project: Project, arguments: List<Any>): CompletableFuture<JsonObject> =
            LanguageServerManager.getInstance(project).getLanguageServer("cpl").thenCompose {
                it!!.server.workspaceService.executeCommand(ExecuteCommandParams("cpl.exec", arguments))
            }.thenApply { Gson().toJsonTree(it).asJsonObject }
    }
}

/**
 * Import a Problem: `cpl lsp`'s `cpl.import` waits for Competitive Companion, creates each problem
 * in the selected folder (else the project's) and opens it through `window/showDocument`.
 */
class CplImportProblemAction : AnAction() {
    override fun getActionUpdateThread() = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.getRequiredData(CommonDataKeys.PROJECT)
        val path = e.getData(CommonDataKeys.VIRTUAL_FILE)?.path
        LanguageServerManager.getInstance(project).getLanguageServer("cpl").thenAccept {
            it?.server?.workspaceService?.executeCommand(ExecuteCommandParams("cpl.import", listOf(path)))
        }
    }
}

/**
 * A Trace lens's run: the page `cpl.exec` answers with, as is in JCEF. The page posts each
 * step's and each clicked line, 0-based and checked, through `jbCefQuery`; the editor shows and marks it.
 */
object CplTrace {
    /** Shows the page a Trace lens's run answered with, if it has one. */
    fun open(project: Project, source: File, result: JsonObject) =
        result.get("html")?.takeIf { it.isJsonPrimitive }?.let { page ->
            ApplicationManager.getApplication().invokeLater { show(project, source, page.asString) }
        }

    fun show(project: Project, source: File, page: String) {
        val file = LocalFileSystem.getInstance().refreshAndFindFileByIoFile(source)!!
        val browser = JBCefBrowser()
        var shown: RangeHighlighter? = null
        val router = CefMessageRouter.create(CefMessageRouter.CefMessageRouterConfig("jbCefQuery", "jbCefQueryCancel"))
        router.addHandler(object : CefMessageRouterHandlerAdapter() {
            override fun onQuery(b: CefBrowser, f: CefFrame, id: Long, line: String, persistent: Boolean, done: CefQueryCallback): Boolean {
                ApplicationManager.getApplication().invokeLater({ shown = follow(project, file, line.toInt(), shown) }, project.disposed)
                return true
            }
        }, true)
        browser.jbCefClient.cefClient.addMessageRouter(router)
        Disposer.register(browser) { router.dispose(); shown?.dispose() }
        browser.loadHTML(page)
        val window = ToolWindowManager.getInstance(project).getToolWindow("CP Lang Trace")!!
        window.isAvailable = true
        window.contentManager.findContent("Trace: ${source.name}")?.let { window.contentManager.removeContent(it, true) }
        val content = ContentFactory.getInstance().createContent(browser.component, "Trace: ${source.name}", false)
        content.setDisposer(browser)
        window.contentManager.addContent(content)
        window.contentManager.setSelectedContent(content)
        window.show()
    }

    /** Scrolls to `line` (0-based) of `file` and marks it like the debugger's execution point, in place of `shown`. */
    fun follow(project: Project, file: VirtualFile, line: Int, shown: RangeHighlighter?): RangeHighlighter {
        shown?.dispose()
        val editor = FileEditorManager.getInstance(project).openTextEditor(OpenFileDescriptor(project, file), false)!!
        editor.scrollingModel.scrollTo(LogicalPosition(line, 0), ScrollType.MAKE_VISIBLE)
        return editor.markupModel.addLineHighlighter(DebuggerColors.EXECUTIONPOINT_ATTRIBUTES, line, HighlighterLayer.SELECTION - 1)
    }
}

/** The Trace tool window plugin.xml declares; hidden until a Trace lens's run fills it. */
class CplTraceWindow : ToolWindowFactory, DumbAware {
    override fun shouldBeAvailable(project: Project) = false

    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) = Unit
}
