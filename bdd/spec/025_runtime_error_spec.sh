Include build_tools.sh

Describe "Runtime Errors"
  # NOTE: The error messages absolutely need improvement, but this will prevent regressions in them
  Describe "File Not Found"
    It "doesn't work"
      When run alan-runtime run nothingburger
      The status should eq "2"
      The error should include "File not found:"
    End
  End
End