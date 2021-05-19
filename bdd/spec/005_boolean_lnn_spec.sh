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

        emit stdout toString(true && true) + ' <- \"true && true\"\n';
        emit stdout toString(and(true, false)) + ' <- \"and(true, false)\"\n';
        emit stdout toString(false & true) + ' <- \"false & true\"\n';

        emit stdout toString(true || true) + ' <- \"true || true\"\n';
        emit stdout toString(or(true, false)) + ' <- \"or(true, false)\"\n';
        emit stdout toString(false | true) + ' <- \"false | true\"\n';
        
        emit stdout toString(true ^ true) + ' <- \"true ^ true\"\n';
        emit stdout toString(xor(true, false)) + ' <- \"xor(true, false)\"\n';
        emit stdout toString(false ^ true) + ' <- \"false ^ true\"\n';

        emit stdout toString(!true) + ' <- \"!true\"\n';
        emit stdout toString(not(false)) + ' <- \"not(false)\"\n';

        emit stdout toString(true !& true) + ' <- \"true !& true\"\n';
        emit stdout toString(nand(true, false)) + ' <- \"nand(true, false)\"\n';
        emit stdout toString(false !& true) + ' <- \"false !& true\"\n';

        emit stdout toString(true !| true) + ' <- \"true !| true\"\n';
        emit stdout toString(nor(true, false)) + ' <- \"nor(true, false)\"\n';
        emit stdout toString(false !| true) + ' <- \"false !| true\"\n';
        
        emit stdout toString(true !^ true) + ' <- \"true !^ true\"\n';
        emit stdout toString(xnor(true, false)) + ' <- \"xnor(true, false)\"\n';
        emit stdout toString(false !^ true) + ' <- \"false !^ true\"\n';

        wait(1000);
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
  OUTPUT12="true <- \"true && true\""
  OUTPUT13="false <- \"and(true, false)"
  OUTPUT14="false <- \"false & true\""
  OUTPUT15="true <- \"true || true\""
  OUTPUT16="true <- \"or(true, false)\""
  OUTPUT17="true <- \"false | true\""
  OUTPUT18="false <- \"true ^ true\""
  OUTPUT19="true <- \"xor(true, false)\""
  OUTPUT20="true <- \"false ^ true\""
  OUTPUT21="false <- \"!true\""
  OUTPUT22="true <- \"not(false)\""
  OUTPUT23="false <- \"true !& true\""
  OUTPUT24="true <- \"nand(true, false)\""
  OUTPUT25="true <- \"false !& true\""
  OUTPUT26="false <- \"true !| true\""
  OUTPUT27="false <- \"nor(true, false)\""
  OUTPUT28="false <- \"false !| true\""
  OUTPUT29="true <- \"true !^ true\""
  OUTPUT30="false <- \"xnor(true, false)\""
  OUTPUT31="false <- \"false !^ true\""

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
    The output should include "$OUTPUT12"
    The output should include "$OUTPUT13"
    The output should include "$OUTPUT14"
    The output should include "$OUTPUT15"
    The output should include "$OUTPUT16"
    The output should include "$OUTPUT17"
    The output should include "$OUTPUT18"
    The output should include "$OUTPUT19"
    The output should include "$OUTPUT20"
    The output should include "$OUTPUT21"
    The output should include "$OUTPUT22"
    The output should include "$OUTPUT23"
    The output should include "$OUTPUT24"
    The output should include "$OUTPUT25"
    The output should include "$OUTPUT26"
    The output should include "$OUTPUT27"
    The output should include "$OUTPUT28"
    The output should include "$OUTPUT29"
    The output should include "$OUTPUT30"
    The output should include "$OUTPUT31"
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
    The output should include "$OUTPUT12"
    The output should include "$OUTPUT13"
    The output should include "$OUTPUT14"
    The output should include "$OUTPUT15"
    The output should include "$OUTPUT16"
    The output should include "$OUTPUT17"
    The output should include "$OUTPUT18"
    The output should include "$OUTPUT19"
    The output should include "$OUTPUT20"
    The output should include "$OUTPUT21"
    The output should include "$OUTPUT22"
    The output should include "$OUTPUT23"
    The output should include "$OUTPUT24"
    The output should include "$OUTPUT25"
    The output should include "$OUTPUT26"
    The output should include "$OUTPUT27"
    The output should include "$OUTPUT28"
    The output should include "$OUTPUT29"
    The output should include "$OUTPUT30"
    The output should include "$OUTPUT31"
  End
End
