package dev.cplang.ide

import com.intellij.execution.process.CapturingProcessHandler
import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.editor.LogicalPosition
import com.intellij.openapi.editor.ScrollType
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.RangeHighlighter
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.OpenFileDescriptor
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.wm.ToolWindowAnchor
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

/**
 * `cpl lsp` through LSP4IJ, wherever LSP4IJ is loaded: `cpl setup jetbrains` installs it
 * in every IDE, because the free IntelliJ IDEA ships the built-in client's classes without
 * the module that turns it on. The built-in client runs `cpl lsp` only where LSP4IJ is absent.
 */
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

        private val id = PluginId.getId("com.redhat.devtools.lsp4ij")

        val loaded: Boolean
            get() = PluginManagerCore.getPlugin(id)?.let { !PluginManagerCore.isDisabled(id) } ?: false
    }
}

/** `cpl.exec`, the command of `cpl lsp`'s code lenses: LSP4IJ runs the action of that id. */
class CplExecAction : LSPCommandAction() {
    override fun getCommandPerformedThread() = ActionUpdateThread.EDT

    override fun commandPerformed(command: LSPCommand, e: AnActionEvent) {
        val project = e.project ?: return
        CplExec.run(project, command.arguments.indices.mapNotNull { command.getArgumentAt<String>(it, String::class.java) })
    }
}

/**
 * A Trace lens's run: `cpl run --trace --format html`'s page as is in JCEF. The page posts each
 * step's and each clicked line, 0-based and checked, through `jbCefQuery`; the editor shows and marks it.
 */
object CplTrace {
    fun open(project: Project, source: File, args: List<String>) = object : Task.Backgroundable(project, "cpl run --trace") {
        override fun run(indicator: ProgressIndicator) {
            val out = CapturingProcessHandler(Cpl.commandLine(args + listOf("--format", "html"), source.parentFile)).runProcess()
            ApplicationManager.getApplication().invokeLater { show(project, source, out.stdout + out.stderr) }
        }

        override fun onThrowable(error: Throwable) = NotificationGroupManager.getInstance().getNotificationGroup("CP Lang")
            .createNotification("No trace", error.message.orEmpty(), NotificationType.ERROR).notify(project)
    }.queue()

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
        val window = ToolWindowManager.getInstance(project).getToolWindow("CP Lang Trace")
            ?: ToolWindowManager.getInstance(project).registerToolWindow("CP Lang Trace") { anchor = ToolWindowAnchor.RIGHT; canCloseContent = true }
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
