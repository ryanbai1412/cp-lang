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
                .comment("Empty: \$CPL, then cpl on the PATH, then ~/.cpl/bin/cpl, then ~/.cargo/bin/cpl.")
        }
    }
}

/** What the plugin runs: every feature is a `cpl` subcommand, so the IDE and the terminal agree. */
object Cpl {
    val COMMANDS = listOf("run", "test", "stress")

    fun executable(): String? {
        CplSettings.get().state.path?.takeIf { it.isNotBlank() }?.let { return it }
        System.getenv("CPL")?.takeIf { it.isNotBlank() }?.let { return it }
        PathEnvironmentVariableUtil.findInPath("cpl")?.let { return it.path }
        val windows = System.getProperty("os.name").startsWith("Windows")
        return installed(System.getProperty("user.home"), System.getenv("CPL_HOME"), windows)
            .firstOrNull { it.canExecute() }?.path
    }

    /**
     * Where the installer (`$CPL_HOME/bin`, by default `~/.cpl/bin`) and cargo put `cpl`, in
     * order: an IDE started from the dock does not see the shell's PATH.
     */
    fun installed(home: String, cplHome: String?, windows: Boolean): List<File> {
        val name = if (windows) "cpl.exe" else "cpl"
        return listOfNotNull(cplHome?.takeIf { it.isNotBlank() }?.let { File(it, "bin") }, File(home, ".cpl/bin"), File(home, ".cargo/bin"))
            .map { File(it, name) }
    }

    fun commandLine(args: List<String>, workDirectory: File?): GeneralCommandLine {
        val exe = executable()
            ?: throw ExecutionException("cpl was not found: set its path in Settings | Tools | CP Lang, or put it on the PATH")
        return GeneralCommandLine(exe)
            .withParameters(args)
            .withWorkDirectory(workDirectory)
            .withCharset(Charsets.UTF_8)
    }

    fun lspArgs() = listOf("lsp")

    /** `cpl run FILE [INPUT]`, `cpl test FILE`, `cpl stress FILE`, then the user's own arguments. */
    fun runArgs(command: String, file: String, input: String?, extra: String): List<String> {
        val args = mutableListOf(command, file)
        if (command == "run" && !input.isNullOrBlank()) args += input
        args += splitArgs(extra)
        return args
    }

    fun traceArgs(file: String, fn: String?) =
        listOf("run", file, "--trace") + listOfNotNull(fn) + listOf("--format", "html")

    fun splitArgs(text: String): List<String> =
        Regex("\"([^\"]*)\"|(\\S+)").findAll(text).map { it.groupValues[1].ifEmpty { it.groupValues[2] } }.toList()

    /**
     * The sample inputs of `DIR/STEM.cp` in the order the VS Code extension shows them:
     * scratch `i1`, `i2`, ... beside the file, then `STEM.tests/N.in`, then `STEM.samples/N.in`.
     */
    fun samples(source: File): List<File> {
        val dir = source.absoluteFile.parentFile ?: return emptyList()
        val stem = source.nameWithoutExtension
        val byNumber = compareBy<File>({ it.name.filter(Char::isDigit).toLongOrNull() ?: Long.MAX_VALUE }, { it.name })
        val scratch = dir.listFiles { f -> f.isFile && Regex("i\\d+").matches(f.name) }.orEmpty().sortedWith(byNumber)
        val cases = listOf("$stem.tests", "$stem.samples").flatMap { sub ->
            File(dir, sub).listFiles { f -> f.isFile && f.name.endsWith(".in") }.orEmpty().sortedWith(byNumber)
        }
        return scratch + cases
    }

    /** Each source line `cpl` names in a line of its output (`at line 4`, `line 5: k: 1`): where, and the 1-based line. */
    fun lineRefs(text: String): List<Pair<IntRange, Int>> =
        listOf(Regex("\\bat (line (\\d+))\\b"), Regex("^\\s+(line (\\d+)):")).flatMap { re ->
            re.findAll(text).map { m -> m.groups[1]!!.range to m.groupValues[2].toInt() }.toList()
        }.sortedBy { it.first.first }

    data class WrongCall(val line: Int, val message: String)

    /**
     * The solution's line of the first wrong call `cpl stress` printed (`solution tri(k: 1) = 0 at line 4`
     * under `first wrong call: ...`), and that headline; the same report Neovim and VS Code draw.
     */
    fun wrongCall(lines: List<String>): WrongCall? {
        val head = Regex("^\\s*((first wrong call|first differing value): .*?)\\s*$")
        val at = Regex("^\\s+solution\\s.*\\sat line (\\d+)\\s*$")
        var headline: String? = null
        for (line in lines) {
            head.find(line)?.let { headline = it.groupValues[1] }
            val h = headline ?: continue
            at.find(line)?.let { return WrongCall(it.groupValues[1].toInt(), h) }
        }
        return null
    }

    /** The top-level `fn` the offset is in, for `cpl run --trace FN`. */
    fun enclosingFn(text: CharSequence, offset: Int): String? {
        val fn = Regex("^fn\\s+([A-Za-z_][A-Za-z0-9_]*)", RegexOption.MULTILINE)
        val top = Regex("^\\S", RegexOption.MULTILINE)
        val head = fn.findAll(text).lastOrNull { it.range.first <= offset } ?: return null
        val next = top.findAll(text, head.range.last + 1).firstOrNull { m ->
            val line = text.subSequence(m.range.first, text.indexOf('\n', m.range.first).let { if (it < 0) text.length else it })
            !line.startsWith("//") && !line.startsWith("@")
        }
        if (next != null && next.range.first <= offset) return null
        return head.groupValues[1]
    }
}
