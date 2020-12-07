Include build_tools.sh

Describe "Runtime Errors"
  # NOTE: The error messages absolutely need improvement, but this will prevent regressions in them
  Describe "File Not Found"
    It "doesn't work"
      When run alan run nothingburger
      The status should eq "2"
      The error should include "File not found:"
    End
  End

  Describe "getOrExit"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const xs = [0, 1, 2, 5];
          const x1 = xs[1].getOrExit();
          print(x1);
          const x2 = xs[2].getOrExit();
          print(x2);
          const x5 = xs[5].getOrExit();
          print(x5);

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "halts js on an error"
      When run test_js
      The status should eq "1"
      The output should include ""
      The error should eq "out-of-bounds access" # TODO: errors need stacktrace-like reporting
    End 

    It "halts agc on an error"
      When run test_agc
      The status should eq "1"
      The error should eq "out-of-bounds access" # TODO: errors need stacktrace-like reporting
    End 
  End
End