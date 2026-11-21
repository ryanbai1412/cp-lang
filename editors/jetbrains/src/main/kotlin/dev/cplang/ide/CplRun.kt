package dev.cplang.ide

import com.intellij.execution.Executor
import com.intellij.execution.actions.ConfigurationContext
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
import com.intellij.execution.process.ProcessHandler
import com.intellij.execution.process.ProcessHandlerFactory
import com.intellij.execution.process.ProcessTerminatedListener
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.icons.AllIcons
import com.intellij.openapi.options.SettingsEditor
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.ComboBox
import com.intellij.openapi.util.Ref
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiWhiteSpace
import com.intellij.psi.util.PsiTreeUtil
import com.intellij.psi.util.elementType
import com.intellij.ui.components.JBTextField
import com.intellij.util.ui.FormBuilder
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
        return Cpl.commandLine(args, file.absoluteFile.parentFile)
    }

    override fun getConfigurationEditor(): SettingsEditor<out RunConfiguration> = CplSettingsEditor()

    override fun getState(executor: Executor, environment: ExecutionEnvironment): RunProfileState =
        object : CommandLineState(environment) {
            override fun startProcess(): ProcessHandler {
                val handler = ProcessHandlerFactory.getInstance().createColoredProcessHandler(commandLine())
                ProcessTerminatedListener.attach(handler)
                return handler
            }
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

class CplRunLineMarkerContributor : RunLineMarkerContributor() {
    override fun getInfo(element: PsiElement): Info? {
        if (element.elementType != CpTokens.KEYWORD || element.text != "fn") return null
        if (element.textOffset > 0 && element.containingFile.text[element.textOffset - 1] != '\n') return null
        val name = PsiTreeUtil.nextLeaf(element)?.let { if (it is PsiWhiteSpace) PsiTreeUtil.nextLeaf(it) else it }
        if (name?.text != "main") return null
        return Info(AllIcons.RunConfigurations.TestState.Run, ExecutorAction.getActions(0)) { "Run ${element.containingFile.name}" }
    }
}
