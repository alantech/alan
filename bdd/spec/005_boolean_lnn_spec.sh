Include build_tools.sh

Describe "Booleans"
  before() {
    lnn_sourceToAll "
      from @std/app import start, stdout, exit

      on start {
        emit stdout concat(toString(true), '\n');
        emit stdout concat(toString(false), '\n');
        emit stdout concat(toString(toBool(1)), '\n');
        emit stdout concat(toString(toBool(0)), '\n');
        emit stdout concat(toString(toBool(15)), '\n');
        emit stdout concat(toString(toBool(-1)), '\n');
        emit stdout concat(toString(toBool(0.0)), '\n');
        emit stdout concat(toString(toBool(1.2)), '\n');
        emit stdout concat(toString(toBool('')), '\n');
        emit stdout concat(toString(toBool('hi')), '\n');
        emit exit 0;
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
