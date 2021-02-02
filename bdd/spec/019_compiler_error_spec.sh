Include build_tools.sh

Describe "Compiler Errors"
  # NOTE: The error messages absolutely need improvement, but this will prevent regressions in them
  Describe "Cross-type comparisons"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          print(true == 1);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan compile test_$$/temp.ln test_$$/temp.amm
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
          return 'blah';
          print('unreachable!');
        }

        on start {
          unreachable();
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan compile test_$$/temp.ln test_$$/temp.amm
      The status should not eq "0"
      # TODO: What file?
      The error should eq "Unreachable code in function 'unreachable' after:
return 'blah'; on line 4:12"
    End
  End

  Describe "Recursive Function calls"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        fn fibonacci(n: int64) {
          if n < 2 {
            return 1;
          } else {
            return fibonacci(n - 1 || 0) + fibonacci(n - 2 || 0);
          }
        }

        on start {
          print(fibonacci(0));
          print(fibonacci(1));
          print(fibonacci(2));
          print(fibonacci(3));
          print(fibonacci(4));
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan compile test_$$/temp.ln test_$$/temp.amm
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
          print(i64str(5)); // Illegal direct opcode usage
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan compile test_$$/temp.ln test_$$/temp.amm
      The status should not eq "0"
      # TODO: What file, line, and character?
      The error should eq "i64str is not a function but used as one.
i64str on line 4:18"
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
      When run alan compile test_$$/temp.ln test_$$/temp.amm
      The status should not eq "0"
      The error should include "Could not load"
      The error should include "No match for OneOrMore (whitespace | exportsn | handlers | functions | types | constdeclaration | operatormapping | events | interfaces) in file fakeFile line 3:10"
    End
  End

  Describe "Importing unexported values"
    before() {
      sourceToFile piece.ln "
        type Piece {
          owner: bool
        }
      "
      sourceToTemp "
        from @std/app import start, print, exit
        from ./piece import Piece

        on start {
          const piece = new Piece {
            owner: false
          };
          print('Hello World');
          if piece.owner == true {
            print('OK');
          } else {
            print('False');
          }
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanFile piece.ln
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan compile test_$$/temp.ln test_$$/temp.amm
      The status should not eq "0"
      The error should eq "Piece is not a type
new Piece {
            owner: false
          } on line 2:26"
    End
  End
End
