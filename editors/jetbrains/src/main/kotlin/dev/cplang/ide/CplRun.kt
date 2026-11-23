package dev.cplang.ide

import com.intellij.execution.Executor
import com.intellij.execution.actions.ConfigurationContext
import com.intellij.execution.filters.Filter
import com.intellij.execution.filters.OpenFileHyperlinkInfo
import com.intellij.execution.actions.LazyRunConfigurationProducer
import com.intellij.execution.configurations.CommandLineState
import com.intellij.execution.configurations.ConfigurationFactory
import com.intellij.execution.configurations.ConfigurationType
import com.intellij.execution.configurations.ConfigurationTypeBase
import com.intellij.execution.configurations.ConfigurationTypeUtil
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.configurations.LocatableConfigurationBase
import com.intellij.execution.configurations.LocatableRunConfigurationOptions
import com.intellij.execution.configurations.RunConfiguration
import com.intellij.execution.configurations.RunProfileState
import com.intellij.execution.configurations.RuntimeConfigurationError
import com.intellij.execution.lineMarker.ExecutorAction
import com.intellij.execution.lineMarker.RunLineMarkerContributor
import com.intellij.execution.process.ProcessAdapter
import com.intellij.execution.process.ProcessEvent
import com.intellij.execution.process.ProcessHandler
import com.intellij.execution.process.ProcessHandlerFactory
import com.intellij.execution.process.ProcessOutputTypes
import com.intellij.execution.process.ProcessTerminatedListener
import com.intellij.execution.testframework.TestConsoleProperties
import com.intellij.execution.testframework.sm.FileUrlProvider
import com.intellij.execution.testframework.sm.SMCustomMessagesParsing
import com.intellij.execution.testframework.sm.SMTestRunnerConnectionUtil
import com.intellij.execution.testframework.sm.runner.OutputToGeneralTestEventsConverter
import com.intellij.execution.testframework.sm.runner.SMTRunnerConsoleProperties
import com.intellij.execution.ui.ConsoleView
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.icons.AllIcons
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.editor.colors.CodeInsightColors
import com.intellij.openapi.editor.EditorCustomElementRenderer
import com.intellij.openapi.editor.Inlay
import com.intellij.openapi.editor.colors.EditorFontType
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.TextEditor
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.util.Key
import com.intellij.ui.JBColor
import com.intellij.openapi.options.SettingsEditor
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.ui.ComboBox
import com.intellij.openapi.util.Ref
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiWhiteSpace
import com.intellij.psi.util.PsiTreeUtil
import com.intellij.psi.util.elementType
import com.intellij.ui.components.JBTextField
import com.intellij.util.ui.FormBuilder
import java.awt.Graphics
import java.awt.Rectangle
import java.io.File
import javax.swing.JComponent

class CplRunConfigurationType : ConfigurationTypeBase(
    ID,
    "CP Lang",
    "Run, test or stress a CP Lang file with cpl",
    CpIcons.FILE,
) {
    init {
        addFactory(CplConfigurationFactory(this))
    }

    companion object {
        const val ID = "CplRunConfiguration"
        fun get(): CplRunConfigurationType = ConfigurationTypeUtil.findConfigurationType(CplRunConfigurationType::class.java)
    }
}

class CplConfigurationFactory(type: ConfigurationType) : ConfigurationFactory(type) {
    override fun getId() = "cpl"
    override fun createTemplateConfiguration(project: Project): RunConfiguration = CplRunConfiguration(project, this, "CP Lang")
    override fun getOptionsClass() = CplRunOptions::class.java
}

class CplRunOptions : LocatableRunConfigurationOptions() {
    var file by string("")
    var command by string("run")
    var input by string("")
    var arguments by string("")
}

class CplRunConfiguration(project: Project, factory: ConfigurationFactory, name: String) :
    LocatableConfigurationBase<CplRunOptions>(project, factory, name) {

    public override fun getOptions() = super.getOptions() as CplRunOptions

    override fun suggestedName(): String? {
        val file = options.file?.takeIf { it.isNotBlank() } ?: return null
        return "${options.command ?: "run"} ${File(file).name}"
    }

    override fun checkConfiguration() {
        if (options.file.isNullOrBlank()) throw RuntimeConfigurationError("Choose a .cp file")
        if (options.command !in Cpl.COMMANDS) throw RuntimeConfigurationError("The command is one of ${Cpl.COMMANDS.joinToString()}")
        Cpl.executable() ?: throw RuntimeConfigurationError("cpl was not found: set its path in Settings | Tools | CP Lang")
    }

    fun commandLine(): GeneralCommandLine {
        val file = File(options.file ?: "")
        val args = Cpl.runArgs(options.command ?: "run", file.path, options.input, options.arguments ?: "")
        // `cpl test` fills the test tree from its JSON report.
        val report = if (options.command == "test") listOf("--format", "json") else emptyList()
        return Cpl.commandLine(args + report, file.absoluteFile.parentFile)
    }

    override fun getConfigurationEditor(): SettingsEditor<out RunConfiguration> = CplSettingsEditor()

    override fun getState(executor: Executor, environment: ExecutionEnvironment): RunProfileState =
        object : CommandLineState(environment) {
            init {
                addConsoleFilters(CplLineFilter(project, File(options.file ?: "").absoluteFile))
            }

            override fun createConsole(executor: Executor): ConsoleView? =
                if (options.command == "test") {
                    SMTestRunnerConnectionUtil.createConsole("CP Lang", CplTestConsoleProperties(this@CplRunConfiguration, executor))
                } else {
                    super.createConsole(executor)
                }

            override fun startProcess(): ProcessHandler {
                val handler = ProcessHandlerFactory.getInstance().createColoredProcessHandler(commandLine())
                ProcessTerminatedListener.attach(handler)
                CplWrongCall.follow(project, File(options.file ?: "").absoluteFile, handler)
                return handler
            }
        }
}

/** The test tree of a `cpl test` run: each case of [Cpl.testsDir], a wrong answer as a diff. */
class CplTestConsoleProperties(private val config: CplRunConfiguration, executor: Executor) :
    SMTRunnerConsoleProperties(config, "CP Lang", executor), SMCustomMessagesParsing {
    override fun getTestLocator() = FileUrlProvider.INSTANCE

    override fun createTestEventsConverter(name: String, properties: TestConsoleProperties): OutputToGeneralTestEventsConverter {
        val source = File(config.options.file ?: "").absoluteFile
        return CplTestEventsConverter(name, properties, Cpl.testsDir(source))
    }
}

/** Holds back `cpl test`'s JSON until it exits, then replays it as test events. */
class CplTestEventsConverter(name: String, properties: TestConsoleProperties, private val tests: File) :
    OutputToGeneralTestEventsConverter(name, properties) {
    private val report = StringBuilder()

    override fun process(text: String, outputType: Key<*>) {
        if (outputType == ProcessOutputTypes.STDOUT) report.append(text) else super.process(text, outputType)
    }

    override fun flushBufferOnProcessTermination(code: Int) {
        val messages = Cpl.testMessages(report.toString(), tests)
        if (messages == null) {
            super.process(report.toString(), ProcessOutputTypes.STDOUT)
        } else {
            messages.forEach { super.process(it + "\n", ProcessOutputTypes.STDOUT) }
        }
        super.flushBufferOnProcessTermination(code)
    }
}

class CplSettingsEditor : SettingsEditor<CplRunConfiguration>() {
    private val file = JBTextField()
    private val command = ComboBox(Cpl.COMMANDS.toTypedArray())
    private val input = JBTextField()
    private val arguments = JBTextField()

    override fun resetEditorFrom(configuration: CplRunConfiguration) {
        val o = configuration.options
        file.text = o.file ?: ""
        command.selectedItem = o.command ?: "run"
        input.text = o.input ?: ""
        arguments.text = o.arguments ?: ""
    }

    override fun applyEditorTo(configuration: CplRunConfiguration) {
        val o = configuration.options
        o.file = file.text
        o.command = command.selectedItem as String
        o.input = input.text
        o.arguments = arguments.text
    }

    override fun createEditor(): JComponent = FormBuilder.createFormBuilder()
        .addLabeledComponent("File:", file)
        .addLabeledComponent("Command:", command)
        .addLabeledComponent("Input (run only):", input)
        .addLabeledComponent("Arguments:", arguments)
        .panel
}

/** Right-click a `.cp` file, or its `fn main`, and Run: `cpl run FILE` on its first sample. */
class CplRunConfigurationProducer : LazyRunConfigurationProducer<CplRunConfiguration>() {
    override fun getConfigurationFactory() = CplRunConfigurationType.get().configurationFactories.single()

    override fun setupConfigurationFromContext(
        configuration: CplRunConfiguration,
        context: ConfigurationContext,
        sourceElement: Ref<PsiElement>,
    ): Boolean {
        val file = context.location?.virtualFile?.takeIf { it.extension == "cp" } ?: return false
        configuration.options.file = file.path
        configuration.options.command = "run"
        configuration.options.input = Cpl.samples(File(file.path)).firstOrNull()?.path ?: ""
        configuration.name = configuration.suggestedName() ?: file.name
        return true
    }

    override fun isConfigurationFromContext(configuration: CplRunConfiguration, context: ConfigurationContext): Boolean {
        val file = context.location?.virtualFile ?: return false
        return configuration.options.file == file.path && configuration.options.command == "run"
    }
}

/** Right-click a `.cp` file that has a `STEM.tests/` folder: `cpl test FILE`, in the test tree. */
class CplTestConfigurationProducer : LazyRunConfigurationProducer<CplRunConfiguration>() {
    override fun getConfigurationFactory() = CplRunConfigurationType.get().configurationFactories.single()

    override fun setupConfigurationFromContext(
        configuration: CplRunConfiguration,
        context: ConfigurationContext,
        sourceElement: Ref<PsiElement>,
    ): Boolean {
        val file = context.location?.virtualFile?.takeIf { it.extension == "cp" } ?: return false
        if (file.parent?.findChild(file.nameWithoutExtension + ".tests")?.isDirectory != true) return false
        configuration.options.file = file.path
        configuration.options.command = "test"
        configuration.name = configuration.suggestedName() ?: file.name
        return true
    }

    override fun isConfigurationFromContext(configuration: CplRunConfiguration, context: ConfigurationContext): Boolean {
        val file = context.location?.virtualFile ?: return false
        return configuration.options.file == file.path && configuration.options.command == "test"
    }
}

class CplRunLineMarkerContributor : RunLineMarkerContributor() {
    override fun getInfo(element: PsiElement): Info? {
        if (element.elementType != CpTokens.KEYWORD || element.text != "fn") return null
        if (element.textOffset > 0 && element.containingFile.text[element.textOffset - 1] != '\n') return null
        val name = PsiTreeUtil.nextLeaf(element)?.let { if (it is PsiWhiteSpace) PsiTreeUtil.nextLeaf(it) else it }
        if (name?.text != "main") return null
        return Info(AllIcons.RunConfigurations.TestState.Run, ExecutorAction.getActions(0)) { "Run ${element.containingFile.name}" }
    }
}

/**
 * Links each source line `cpl` names in its output (`solution tri(k: 1) = 0 at line 4` under
 * `cpl stress`'s first wrong call, `line 5: k: 1` in its last steps) to that line of the file.
 */
class CplLineFilter(private val project: Project, private val file: File) : Filter {
    override fun applyFilter(line: String, entireLength: Int): Filter.Result? {
        val refs = Cpl.lineRefs(line).takeIf { it.isNotEmpty() } ?: return null
        val vfile = LocalFileSystem.getInstance().findFileByIoFile(file) ?: return null
        val start = entireLength - line.length
        return Filter.Result(refs.map { (range, n) ->
            Filter.ResultItem(start + range.first, start + range.last + 1, OpenFileHyperlinkInfo(project, vfile, n - 1))
        })
    }
}

/**
 * `cpl stress`'s first wrong call drawn on that line of the file, as VS Code and Neovim do: a warning
 * on the line with the headline after it, until the next run of the file.
 */
object CplWrongCall {
    private val shown = mutableMapOf<String, List<() -> Unit>>()

    fun follow(project: Project, file: File, handler: ProcessHandler) {
        val out = StringBuffer()
        later(project) { show(project, file, null) }
        handler.addProcessListener(object : ProcessAdapter() {
            override fun onTextAvailable(event: ProcessEvent, outputType: Key<*>) {
                out.append(event.text)
            }

            override fun processTerminated(event: ProcessEvent) {
                val found = Cpl.wrongCall(out.toString().lines())
                later(project) { show(project, file, found) }
            }
        })
    }

    private fun later(project: Project, action: () -> Unit) =
        ApplicationManager.getApplication().invokeLater { if (!project.isDisposed) action() }

    fun show(project: Project, file: File, found: Cpl.WrongCall?) {
        shown.remove(file.path)?.forEach { it() }
        found ?: return
        val vfile = LocalFileSystem.getInstance().findFileByIoFile(file) ?: return
        shown[file.path] = FileEditorManager.getInstance(project).getAllEditors(vfile)
            .filterIsInstance<TextEditor>().map { it.editor }
            .filter { found.line in 1..it.document.lineCount }
            .flatMap { editor ->
                val line = found.line - 1
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
