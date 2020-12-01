Include build_tools.sh

Describe "@std/cmd"
  Describe "exec"
    before() {
      sourceToAll "
        import @std/app
        import @std/cmd

        on app.start {
          const executionResult: cmd.ExecRes = cmd.exec('echo 1')
          app.print(executionResult.stdout)
          emit app.exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    EXECOUTPUT="1"

    It "runs js"
      When run test_js
      The output should start with "$EXECOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should start with "$EXECOUTPUT"
    End
  End
End