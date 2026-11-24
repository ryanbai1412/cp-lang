package dev.cplang.ide

import com.intellij.execution.Executor
import com.intellij.execution.configurations.CommandLineState
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.configurations.RunProfile
import com.intellij.execution.configurations.RunProfileState
import com.intellij.execution.executors.DefaultRunExecutor
import com.intellij.execution.process.ColoredProcessHandler
import com.intellij.execution.process.KillableProcessHandler
import com.intellij.execution.process.ProcessHandler
import com.intellij.execution.process.ProcessOutputTypes
import com.intellij.execution.process.ProcessTerminatedListener
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.execution.runners.ExecutionEnvironmentBuilder
import com.intellij.execution.testframework.sm.FileUrlProvider
import com.intellij.execution.testframework.sm.SMTestRunnerConnectionUtil
import com.intellij.execution.testframework.sm.runner.SMTRunnerConsoleProperties
import com.intellij.execution.ui.ConsoleView
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.editor.EditorCustomElementRenderer
import com.intellij.openapi.editor.Inlay
import com.intellij.openapi.editor.colors.CodeInsightColors
import com.intellij.openapi.editor.colors.EditorFontType
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.TextEditor
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.util.Key
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.ui.JBColor
import java.awt.Graphics
import java.awt.Rectangle
import java.io.File

/**
 * `cpl.exec`, the command of `cpl lsp`'s Run, Trace, Test and Stress code lenses: `cpl ARGS` in the
 * Run tool window. `cpl test` fills the test tree from its TeamCity messages, `cpl stress` shows its
 * JSON report's view, and a traced run opens the trace viewer.
 */
object CplExec {
    fun run(project: Project, args: List<String>) {
        val file = File(args.getOrNull(1) ?: return).absoluteFile
        FileDocumentManager.getInstance().saveAllDocuments()
        if ("--trace" in args) {
            return CplTrace.open(project, file, null, args.getOrNull(2)?.takeIf { it != "--trace" }?.let(::File))
        }
        ExecutionEnvironmentBuilder.create(project, DefaultRunExecutor.getRunExecutorInstance(), Profile(args, file))
            .buildAndExecute()
    }

    /** `args` with the report format the Run window reads. */
    fun commandLine(args: List<String>, file: File): GeneralCommandLine {
        val format = when (args.firstOrNull()) {
            "test" -> listOf("--format", "teamcity")
            "stress" -> listOf("--format", "json")
            else -> emptyList()
        }
        return Cpl.commandLine(args + format, file.parentFile)
    }

    class Profile(val args: List<String>, val file: File) : RunProfile {
        override fun getName() = "cpl ${args[0]} ${file.name}"

        override fun getIcon() = CpIcons.FILE

        override fun getState(executor: Executor, environment: ExecutionEnvironment): RunProfileState =
            object : CommandLineState(environment) {
                override fun createConsole(executor: Executor): ConsoleView? =
                    if (args[0] == "test") {
                        val properties = object : SMTRunnerConsoleProperties(environment.project, this@Profile, "CP Lang", executor) {
                            override fun getTestLocator() = FileUrlProvider.INSTANCE
                        }
                        SMTestRunnerConnectionUtil.createConsole("CP Lang", properties)
                    } else {
                        super.createConsole(executor)
                    }

                override fun startProcess(): ProcessHandler {
                    val cmd = commandLine(args, file)
                    val handler = if (args[0] == "stress") CplWrongCall.Handler(environment.project, file, cmd) else ColoredProcessHandler(cmd)
                    ProcessTerminatedListener.attach(handler)
                    return handler
                }
            }
    }
}
/**
 * The first wrong call `cpl stress`'s report view marks, drawn on that line of the file as VS Code and
 * Neovim do: a warning on the line with the headline after it, until the next run of the file.
 */
object CplWrongCall {
    private val shown = mutableMapOf<String, List<() -> Unit>>()

    /**
     * Runs a command whose stdout is a JSON report with a `view`: the console shows its panel and the
     * file its first wrong call.
     */
    class Handler(private val project: Project, private val file: File, cmd: GeneralCommandLine) : KillableProcessHandler(cmd) {
        private val json = StringBuffer()

        init {
            later(project) { show(project, file, null) }
        }

        override fun notifyTextAvailable(text: String, outputType: Key<*>) {
            if (outputType == ProcessOutputTypes.STDOUT) json.append(text) else super.notifyTextAvailable(text, outputType)
        }

        override fun notifyProcessTerminated(exitCode: Int) {
            val view = Cpl.view(json.toString())
            super.notifyTextAvailable(view?.panel ?: json.toString(), ProcessOutputTypes.STDOUT)
            later(project) { show(project, file, view?.wrongCall) }
            super.notifyProcessTerminated(exitCode)
        }
    }

    private fun later(project: Project, action: () -> Unit) =
        ApplicationManager.getApplication().invokeLater { if (!project.isDisposed) action() }

    fun show(project: Project, file: File, found: Cpl.WrongCall?) {
        shown.remove(file.path)?.forEach { it() }
        found ?: return
        val vfile = LocalFileSystem.getInstance().findFileByIoFile(file) ?: return
        shown[file.path] = FileEditorManager.getInstance(project).getAllEditors(vfile)
            .filterIsInstance<TextEditor>().map { it.editor }
            .filter { found.line in 0 until it.document.lineCount }
            .flatMap { editor ->
                val line = found.line
                val mark = editor.markupModel.addLineHighlighter(CodeInsightColors.WARNINGS_ATTRIBUTES, line, HighlighterLayer.WARNING)
                mark.errorStripeTooltip = found.message
                val note = editor.inlayModel.addAfterLineEndElement(editor.document.getLineEndOffset(line), true, Note("← ${found.message}"))
                listOfNotNull({ mark.dispose() }, note?.let { { Disposer.dispose(it) } })
            }
    }

    class Note(val text: String) : EditorCustomElementRenderer {
        private fun font(inlay: Inlay<*>) = inlay.editor.colorsScheme.getFont(EditorFontType.ITALIC)

        override fun calcWidthInPixels(inlay: Inlay<*>) =
            inlay.editor.contentComponent.getFontMetrics(font(inlay)).stringWidth(text) + 16

        override fun paint(inlay: Inlay<*>, g: Graphics, r: Rectangle, textAttributes: TextAttributes) {
            g.font = font(inlay)
            g.color = JBColor.GRAY
            g.drawString(text, r.x + 8, r.y + g.fontMetrics.ascent)
        }
    }
}
