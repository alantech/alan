Include build_tools.sh

Describe "Events"
  Describe "normal exit code"
    before() {
      sourceToAll "
        from @std/app import start, exit

        on start { emit exit 0 }
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
      sourceToAll "
        from @std/app import start, exit

        on start { emit exit 1 }
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
          let x: int64 = 0
          emit app.exit x
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
End
