Include build_tools.sh

Describe "Compiler Errors"
  # NOTE: The error messages absolutely need improvement, but this will prevent regressions in them
  Describe "Cross-type comparisons"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(true == 1)
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan-compile temp.ln temp.amm
      The status should not eq "0"
      # TODO: What file, line and character?
      The error should eq "Cannot resolve operators with remaining statement
true == 1
<bool> == <int64>"
    End
  End

  Describe "Unreachable code"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        fn unreachable() {
          return 'blah'
          print('unreachable!')
        }

        on start {
          unreachable()
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan-compile temp.ln temp.amm
      The status should not eq "0"
      # TODO: What file?
      The error should eq "Unreachable code in function 'unreachable' after:
return 'blah' on line 5:10"
    End
  End
End
