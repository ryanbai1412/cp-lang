package dev.cplang.ide

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.extapi.psi.PsiFileBase
import com.intellij.lang.Language
import com.intellij.openapi.components.BaseState
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.SimplePersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.openapi.components.service
import com.intellij.openapi.fileTypes.LanguageFileType
import com.intellij.openapi.options.BoundConfigurable
import com.intellij.openapi.util.IconLoader
import com.intellij.psi.FileViewProvider
import com.intellij.ui.dsl.builder.COLUMNS_LARGE
import com.intellij.ui.dsl.builder.bindText
import com.intellij.ui.dsl.builder.columns
import com.intellij.ui.dsl.builder.panel
import com.intellij.ui.dsl.builder.toNonNullableProperty
import java.io.File
import javax.swing.Icon

object CpLanguage : Language("CP") {
    private fun readResolve(): Any = CpLanguage
    override fun getDisplayName() = "CP Lang"
}

object CpIcons {
    @JvmField
    val FILE: Icon = IconLoader.getIcon("/icons/cp.svg", CpIcons::class.java)
}

object CpFileType : LanguageFileType(CpLanguage) {
    override fun getName() = "CP Lang"
    override fun getDescription() = "CP Lang source"
    override fun getDefaultExtension() = "cp"
    override fun getIcon() = CpIcons.FILE
}

class CpFile(viewProvider: FileViewProvider) : PsiFileBase(viewProvider, CpLanguage) {
    override fun getFileType() = CpFileType
    override fun toString() = "CP Lang file"
}

@Service(Service.Level.APP)
@State(name = "CpLangSettings", storages = [Storage("cp-lang.xml")])
class CplSettings : SimplePersistentStateComponent<CplSettings.Options>(Options()) {
    class Options : BaseState() {
        var path by string("")
    }

    companion object {
        fun get(): CplSettings = service()
    }
}

class CplConfigurable : BoundConfigurable("CP Lang") {
    override fun createPanel() = panel {
        val options = CplSettings.get().state
        row("cpl executable:") {
            textField()
                .columns(COLUMNS_LARGE)
                .bindText(options::path.toNonNullableProperty(""))
                .comment("`cpl setup jetbrains` sets it to the cpl it ran as. Empty: cpl on the PATH.")
        }
    }
}

/** What the plugin runs: every feature is a `cpl` subcommand, so the IDE and the terminal agree. */
object Cpl {
    /** The `cpl` Settings names, else the one on the PATH (empty means unset). */
    fun executable(): String = CplSettings.get().state.path.orEmpty().ifBlank { "cpl" }

    fun commandLine(args: List<String>, workDirectory: File?): GeneralCommandLine =
        GeneralCommandLine(executable())
            .withParameters(args)
            .withWorkDirectory(workDirectory)
            .withCharset(Charsets.UTF_8)

    fun lspArgs() = listOf("lsp")
}
