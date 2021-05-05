Include build_tools.sh

Describe "Booleans"
  before() {
    lnn_sourceToAll "
      from @std/app import start, stdout, exit

      on start {
        emit stdout toString(true);
        emit stdout toString(false);
        emit stdout toString(toBool(1));
        emit stdout toString(toBool(0));
        emit stdout toString(toBool(15));
        emit stdout toString(toBool(-1));
        emit stdout toString(toBool(0.0));
        emit stdout toString(toBool(1.2));
        emit stdout toString(toBool(''));
        emit stdout toString(toBool('hi'));
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  OUTPUT="true
false
true
false
true
true
false
true
false
false"

  It "runs js"
    When run test_js
    The output should eq "$OUTPUT"
  End

  It "runs agc"
    When run test_agc
    The output should eq "$OUTPUT"
  End
End
