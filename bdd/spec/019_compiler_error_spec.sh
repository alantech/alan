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
      When run alan compile temp.ln temp.amm
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
      When run alan compile temp.ln temp.amm
      The status should not eq "0"
      # TODO: What file?
      The error should eq "Unreachable code in function 'unreachable' after:
return 'blah' on line 5:10"
    End
  End

  Describe "Recursive Function calls"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        fn fibonacci(n: int64) {
          if n < 2 {
            return 1
          } else {
            return fibonacci(n - 1) + fibonacci(n - 2)
          }
        }

        on start {
          print(fibonacci(0))
          print(fibonacci(1))
          print(fibonacci(2))
          print(fibonacci(3))
          print(fibonacci(4))
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
      When run alan compile temp.ln temp.amm
      The status should not eq "0"
      # TODO: What file, line, and character?
      The error should eq "Recursive callstack detected: fibonacci -> fibonacci. Aborting."
    End
  End

  Describe "Direct opcode calls"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(i64str(5)) // Illegal direct opcode usage
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
      When run alan compile temp.ln temp.amm
      The status should not eq "0"
      # TODO: What file, line, and character?
      The error should eq "Undefined function called: i64str"
    End
  End

  Describe "Totally broken statements"
    before() {
      sourceToTemp "
        import @std/app

        on app.start {
          app.oops
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan compile temp.ln temp.amm
      The status should not eq "0"
      # TODO: Eliminate ANTLR
      The error should eq "line 6:8 extraneous input '}' expecting {'=', NEWLINE, WS}
line 8:0 extraneous input '<EOF>' expecting {'const', 'let', 'return', 'emit', BOOLCONSTANT, 'if', '}', '(', '[', '.', NEWLINE, WS, STRINGCONSTANT, NUMBERCONSTANT, VARNAME}
Cannot read property 'basicassignables' of null"
    End
  End
End
