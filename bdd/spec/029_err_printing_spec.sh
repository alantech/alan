Include build_tools.sh

Describe "Error printing"
  Describe "eprint function"
    before() {
      sourceToAll "
        from @std/app import start, eprint, exit
        on start {
          eprint('This is an error');
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The stderr should eq "This is an error"
    End

    It "runs agc"
      When run test_agc
      The stderr should eq "This is an error"
    End
  End

  Describe "stderr event"
    before() {
      sourceToAll "
        from @std/app import start, stderr, exit
        on start {
          emit stderr 'This is an error';
          wait(10);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The stderr should eq "This is an error"
    End

    It "runs agc"
      When run test_agc
      The stderr should eq "This is an error"
    End
  End
End
