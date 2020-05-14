Include build_tools.sh

Describe "Events"
  before_all() {
    jsRuntime
  }
  BeforeAll before_all

  Describe "normal exit code"
    before() {
      sourceToTemp "
        from @std/app import start, exit

        on start { emit exit 0 }
      "
      tempToAmm
      tempToAgc
      tempToJs
    }
    BeforeAll before

    after() { 
      cleanTemp
    }
    AfterAll after

    It "interprets"
      When run alan-interpreter interpret temp.ln
      The status should eq "0"
    End

    It "interprets IR"
      When run alan-interpreter interpret temp.amm
      The status should eq "0"
    End

    It "runs js"
      When run node temp.js
      The status should eq "0"
    End

    It "runs bc"
      When run alan-runtime run temp.agc
      The status should eq "0"
    End
  End

  Describe "error exit code"
    before() {
      sourceToTemp "
        from @std/app import start, exit

        on start { emit exit 1 }
      "
      tempToAmm
      tempToAgc
      tempToJs
    }
    BeforeAll before

    after() { 
      cleanTemp
    }
    AfterAll after

    It "interprets"
      When run alan-interpreter interpret temp.ln
      The status should eq "1"
    End

    It "interprets IR"
      # Currently fails because the integer type is incorrect (expected int8 got int64)
      Pending correct_type_coersion
      When run alan-interpreter interpret temp.amm
      The status should eq "1"
    End

    It "runs js"
      When run node temp.js
      The status should eq "1"
    End

    It "runs bc"
      # Works because little endian "automatically" coerces to the right value if you can just trim
      When run alan-runtime run temp.agc
      The status should eq "1"
    End
  End
End
