Include build_tools.sh

Describe "Events"
  Describe "normal exit code"
    before() {
      lnn_sourceToAll "
        from @std/app import start, exit

        on start { emit exit 0; }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The status should eq "0"
    End

    It "runs agc"
      When run test_agc
      The status should eq "0"
    End
  End

  Describe "error exit code"
    before() {
      lnn_sourceToAll "
        from @std/app import start, exit

        on start { emit exit 1; }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The status should eq "1"
    End

    It "runs agc"
      # Works because little endian "automatically" coerces to the right value if you can just trim
      When run test_agc
      The status should eq "1"
    End
  End

  Describe "no global memory exit code"
    before() {
      sourceToAll "
        import @std/app

        on app.start {
          let x: int64 = 0;
          emit app.exit x;
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
      The status should eq "0"
    End

    It "runs agc"
      When run test_agc
      The status should eq "0"
    End
  End

  Describe "passing integers from global memory"
    before() {
      # TODO: sourceToAll
      sourceToTemp "
      from @std/app import start, print, exit

      event aNumber: int64;

      on aNumber fn(num: int64) {
        print('I got a number! ' + num.toString());
        emit exit 0;
      }

      on start {
        emit aNumber 5;
      }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after
    INTOUTPUT="I got a number! 5"

    It "runs js"
      Pending function-selection
      When run test_js
      The output should eq "$INTOUTPUT"
    End

    It "runs agc"
      Pending function-selection
      When run test_agc
      The output should eq "$INTOUTPUT"
    End
  End
End
