Include build_tools.sh

Describe "clone"
  before() {
    lnn_sourceToAll "
      from @std/app import start, print, exit

      on start {
        let a = 3;
        let b = a.clone();
        a = 4;
        print(a);
        print(b);
        emit exit 0;
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  CLONEOUTPUT="4
3"

  It "runs js"
    When run test_js
    The output should eq "$CLONEOUTPUT"
  End

  It "runs agc"
    When run test_agc
    The output should eq "$CLONEOUTPUT"
  End
End
