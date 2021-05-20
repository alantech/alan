Include build_tools.sh

Describe "Booleans"
  before() {
    lnn_sourceToAll "
      from @std/app import start, stdout, exit

      on start {
        emit stdout concat(true.toString(), \" <- true\n\");
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
        emit stdout (false & true).toString() + ' <- \"false & true\"\n';
        emit stdout false.and(true).toString() + ' <- \"false.and(true)\"\n';

        emit stdout toString(true || true) + ' <- \"true || true\"\n';
        emit stdout toString(or(true, false)) + ' <- \"or(true, false)\"\n';
        emit stdout (false | true).toString() + ' <- \"false | true\"\n';
        emit stdout false.or(true).toString() + ' <- \"false.or(true)\"\n';
        
        emit stdout toString(true ^ true) + ' <- \"true ^ true\"\n';
        emit stdout toString(xor(true, false)) + ' <- \"xor(true, false)\"\n';
        emit stdout (false ^ true).toString() + ' <- \"false ^ true\"\n';
        emit stdout false.xor(true).toString() + ' <- \"false.xor(true)\"\n';

        emit stdout toString(!true) + ' <- \"!true\"\n';
        emit stdout toString(not(false)) + ' <- \"not(false)\"\n';

        emit stdout toString(true !& true) + ' <- \"true !& true\"\n';
        emit stdout toString(nand(true, false)) + ' <- \"nand(true, false)\"\n';
        emit stdout (false !& true).toString() + ' <- \"false !& true\"\n';
        emit stdout false.nand(true).toString() + ' <- \"false.nand(true)\"\n';

        emit stdout toString(true !| true) + ' <- \"true !| true\"\n';
        emit stdout toString(nor(true, false)) + ' <- \"nor(true, false)\"\n';
        emit stdout (false !| true).toString() + ' <- \"false !| true\"\n';
        emit stdout false.nor(true).toString() + ' <- \"false.nor(true)\"\n';
        
        emit stdout toString(true !^ true) + ' <- \"true !^ true\"\n';
        emit stdout toString(xnor(true, false)) + ' <- \"xnor(true, false)\"\n';
        emit stdout (false !^ true).toString() + ' <- \"false !^ true\"\n';
        emit stdout false.xnor(true).toString() + ' <- \"false.xnor(true)\"\n';

        wait(10);
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
  OUTPUT15="false <- \"false.and(true)\""
  OUTPUT16="true <- \"true || true\""
  OUTPUT17="true <- \"or(true, false)\""
  OUTPUT18="true <- \"false | true\""
  OUTPUT19="true <- \"false.or(true)\""
  OUTPUT20="false <- \"true ^ true\""
  OUTPUT21="true <- \"xor(true, false)\""
  OUTPUT22="true <- \"false ^ true\""
  OUTPUT23="true <- \"false.xor(true)\""
  OUTPUT24="false <- \"!true\""
  OUTPUT25="true <- \"not(false)\""
  OUTPUT26="false <- \"true !& true\""
  OUTPUT27="true <- \"nand(true, false)\""
  OUTPUT28="true <- \"false !& true\""
  OUTPUT29="true <- \"false.nand(true)\""
  OUTPUT30="false <- \"true !| true\""
  OUTPUT31="false <- \"nor(true, false)\""
  OUTPUT32="false <- \"false !| true\""
  OUTPUT33="false <- \"false.nor(true)\""
  OUTPUT34="true <- \"true !^ true\""
  OUTPUT35="false <- \"xnor(true, false)\""
  OUTPUT36="false <- \"false !^ true\""
  OUTPUT37="false <- \"false.xnor(true)\""

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
    The output should include "$OUTPUT33"
    The output should include "$OUTPUT34"
    The output should include "$OUTPUT35"
    The output should include "$OUTPUT36"
    The output should include "$OUTPUT37"
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
    The output should include "$OUTPUT33"
    The output should include "$OUTPUT34"
    The output should include "$OUTPUT35"
    The output should include "$OUTPUT36"
    The output should include "$OUTPUT37"
  End
End
