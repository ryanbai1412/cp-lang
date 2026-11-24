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

    /** `cpl new --listen`: wait for Competitive Companion, then create each problem it sends. */
    fun companionArgs(port: Int) = listOf("new", "--listen", "--json", "--port", port.toString())

    /** The `.cp` files `cpl new --json` created: one JSON object per line. */
    fun createdFiles(stdout: String): List<String> = stdout.lines().mapNotNull { line ->
        runCatching { com.google.gson.JsonParser.parseString(line).asJsonObject.get("path")?.asString }.getOrNull()
    }

    /** The first wrong call a report's `view` marks: its 0-based line and headline. */
    data class WrongCall(val line: Int, val message: String)

    /** What `cpl stress --format json` and `cpl locate --json` say to draw (spec/tools/cli.md). */
    data class View(val panel: String, val wrongCall: WrongCall?)

    fun view(json: String): View? = runCatching {
        val view = com.google.gson.JsonParser.parseString(json).asJsonObject.getAsJsonObject("view")
        val d = view.get("diagnostic")?.takeIf { it.isJsonObject }?.asJsonObject
        View(view.get("panel").asString, d?.let { WrongCall(it.get("line").asInt, it.get("message").asString) })
    }.getOrNull()
}
