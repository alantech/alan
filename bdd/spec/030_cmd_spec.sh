Include build_tools.sh

Describe "@std/cmd"
  Describe "exec"
    before() {
      sourceToAll "
        import @std/app
        import @std/cmd

        on app.start {
          const executionResult: cmd.ExecRes = cmd.exec('echo 1');
          app.print(executionResult.stdout);
          emit app.exit 0;
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

  Describe "exec runs sequentially by default"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/cmd import exec

        on start {
          exec('touch test.txt');
          exec('echo foo >> test.txt');
          exec('echo bar >> test.txt');
          exec('cat test.txt').stdout.print();
          exec('rm test.txt');

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    EXECSEQOUTPUT="foo
bar"
    It "runs js"
      When run test_js
      The output should eq "$EXECSEQOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$EXECSEQOUTPUT"
    End
  End
End