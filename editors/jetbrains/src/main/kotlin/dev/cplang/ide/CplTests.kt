package dev.cplang.ide

import com.google.gson.JsonArray
import com.google.gson.JsonObject
import com.intellij.execution.Executor
import com.intellij.execution.configurations.RunProfile
import com.intellij.execution.executors.DefaultRunExecutor
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.execution.testframework.sm.SMTestRunnerConnectionUtil
import com.intellij.execution.testframework.sm.runner.GeneralToSMTRunnerEventsConvertor
import com.intellij.execution.testframework.sm.runner.SMTRunnerConsoleProperties
import com.intellij.execution.testframework.sm.runner.events.TestFailedEvent
import com.intellij.execution.testframework.sm.runner.events.TestFinishedEvent
import com.intellij.execution.testframework.sm.runner.events.TestStartedEvent
import com.intellij.execution.testframework.sm.runner.ui.SMTRunnerConsoleView
import com.intellij.execution.ui.RunContentDescriptor
import com.intellij.execution.ui.RunContentManager
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import java.io.File

/**
 * A Test lens's cases in the native test tree, read from the `view.tests` its `cpl.exec` answers
 * with: each case's name, outcome, time, message and the expected answer a wrong output is compared with.
 */
object CplTests {
    fun open(project: Project, source: File, result: JsonObject) =
        result.getAsJsonObject("view")?.getAsJsonArray("tests")?.let { tests ->
            ApplicationManager.getApplication().invokeLater { show(project, source, tests) }
        }

    fun show(project: Project, source: File, tests: JsonArray): SMTRunnerConsoleView {
        val executor = DefaultRunExecutor.getRunExecutorInstance()
        val profile = Profile("cpl test ${source.name}")
        val console = SMTestRunnerConnectionUtil.createConsole(
            "CP Lang", SMTRunnerConsoleProperties(project, profile, "CP Lang", executor),
        ) as SMTRunnerConsoleView
        val tree = GeneralToSMTRunnerEventsConvertor(project, console.resultsViewer.testsRootNode, "CP Lang")
        tree.addEventsListener(console.resultsViewer)
        tree.onStartTesting()
        tests.forEach { case -> draw(tree, case.asJsonObject) }
        tree.onFinishTesting()
        RunContentManager.getInstance(project)
            .showRunContent(executor, RunContentDescriptor(console, null, console.component, profile.name))
        return console
    }

    private fun draw(tree: GeneralToSMTRunnerEventsConvertor, case: JsonObject) {
        val name = case.get("name").asString
        tree.onTestStarted(TestStartedEvent(name, null))
        case.get("message")?.let { message ->
            val errored = case.get("outcome").asString == "errored"
            val expected = case.get("expected")?.asString
            tree.onTestFailure(TestFailedEvent(name, message.asString, null, errored, case.get("output").asString, expected))
        }
        tree.onTestFinished(TestFinishedEvent(name, case.get("time_ms").asLong))
    }

    private class Profile(private val title: String) : RunProfile {
        override fun getName() = title

        override fun getIcon() = CpIcons.FILE

        override fun getState(executor: Executor, environment: ExecutionEnvironment) = null
    }
}
