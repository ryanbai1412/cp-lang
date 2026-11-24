package dev.cplang.ide

import com.intellij.execution.ExecutionException
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.configurations.PathEnvironmentVariableUtil
import com.intellij.openapi.components.BaseState
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.SimplePersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.openapi.components.service
import com.intellij.openapi.options.BoundConfigurable
import com.intellij.ui.dsl.builder.bindText
import com.intellij.ui.dsl.builder.columns
import com.intellij.ui.dsl.builder.COLUMNS_LARGE
import com.intellij.ui.dsl.builder.panel
import java.io.File

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
                .bindText({ options.path ?: "" }, { options.path = it })
                .comment("`cpl setup jetbrains` sets it to the cpl it ran as. Empty: cpl on the PATH.")
        }
    }
}

/** What the plugin runs: every feature is a `cpl` subcommand, so the IDE and the terminal agree. */
object Cpl {
    fun executable(): String? =
        CplSettings.get().state.path?.takeIf { it.isNotBlank() } ?: PathEnvironmentVariableUtil.findInPath("cpl")?.path

    fun commandLine(args: List<String>, workDirectory: File?): GeneralCommandLine {
        val exe = executable()
            ?: throw ExecutionException("cpl was not found: set its path in Settings | Tools | CP Lang, or put it on the PATH")
        return GeneralCommandLine(exe)
            .withParameters(args)
            .withWorkDirectory(workDirectory)
            .withCharset(Charsets.UTF_8)
    }

    fun lspArgs() = listOf("lsp")
}
