Include build_tools.sh

Describe "Booleans"
  before() {
    lnn_sourceToAll "
      from @std/app import start, stdout, exit

      on start {
        emit stdout concat(toString(true), \" <- true\n\");
        emit stdout concat(toString(false), \" <- false\n\");
        emit stdout concat(toString(toBool(1)), \" <- 1\n\");
        emit stdout concat(toString(toBool(0)), \" <- 0\n\");
        emit stdout concat(toString(toBool(15)), \" <- 15\n\");
        emit stdout concat(toString(toBool(-1)), \" <- -1\n\");
        emit stdout concat(toString(toBool(0.0)), \" <- 0.0\n\");
        emit stdout concat(toString(toBool(1.2)), \" <- 1.2\n\");
        emit stdout concat(toString(toBool('')), ' <- \"\"\n');
        emit stdout concat(toString(toBool('hi')), \" <- 'hi'\n\");
        emit stdout concat(toString(toBool('true')), \" <- 'true'\n\");
        emit exit 0;
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  OUTPUT1="true <- true"
  OUTPUT2="false <- false"
  OUTPUT3="true <- 1"
  OUTPUT4="false <- 0"
  OUTPUT5="true <- 15"
  OUTPUT6="true <- -1"
  OUTPUT7="false <- 0.0"
  OUTPUT8="true <- 1.2"
  OUTPUT9="false <- \"\""
  OUTPUT10="false <- 'hi'"
  OUTPUT11="true <- 'true'"

  It "runs js"
    When run test_js
    The output should include "$OUTPUT1"
    The output should include "$OUTPUT2"
    The output should include "$OUTPUT3"
    The output should include "$OUTPUT4"
    The output should include "$OUTPUT5"
    The output should include "$OUTPUT6"
    The output should include "$OUTPUT7"
    The output should include "$OUTPUT8"
    The output should include "$OUTPUT9"
    The output should include "$OUTPUT10"
    The output should include "$OUTPUT11"
  End

  It "runs agc"
    When run test_agc
    The output should include "$OUTPUT1"
    The output should include "$OUTPUT2"
    The output should include "$OUTPUT3"
    The output should include "$OUTPUT4"
    The output should include "$OUTPUT5"
    The output should include "$OUTPUT6"
    The output should include "$OUTPUT7"
    The output should include "$OUTPUT8"
    The output should include "$OUTPUT9"
    The output should include "$OUTPUT10"
    The output should include "$OUTPUT11"
  End
End
