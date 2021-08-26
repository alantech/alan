Include build_tools.sh

Describe "Basic Math"
  Describe "int8 (not default)"
    Describe "addition"
      before() {
        lnn_sourceToAll "
          from @std/app import start, exit
          on start { emit exit add(1, 2).getOr(0); }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The status should eq "3"
      End

      It "runs agc"
        When run test_agc
        The status should eq "3"
      End
    End

    Describe "subtraction"
      before() {
        lnn_sourceToAll "
          from @std/app import start, exit
          on start { emit exit sub(2, 1).getOr(0); }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The status should eq "1"
      End

      It "runs agc"
        When run test_agc
        The status should eq "1"
      End
    End

    Describe "multiplication"
      before() {
        lnn_sourceToAll "
          from @std/app import start, exit
          on start { emit exit mul(2, 1).getOr(0); }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The status should eq "2"
      End

      It "runs agc"
        When run test_agc
        The status should eq "2"
      End
    End

    Describe "division"
      before() {
        lnn_sourceToAll "
          from @std/app import start, exit
          on start { emit exit div(6, 2).getOr(0); }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The status should eq "3"
      End

      It "runs agc"
        When run test_agc
        The status should eq "3"
      End
    End

    Describe "modulus"
      before() {
        lnn_sourceToAll "
          from @std/app import start, exit
          on start { emit exit mod(6, 4); }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The status should eq "2"
      End

      It "runs agc"
        When run test_agc
        The status should eq "2"
      End
    End

    Describe "exponentiation"
      before() {
        lnn_sourceToAll "
          from @std/app import start, exit
          on start { emit exit pow(6, 2).getOr(0); }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The status should eq "36"
      End

      It "runs agc"
        When run test_agc
        The status should eq "36"
      End
    End

    Describe "minimum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            min(3.toInt8(), 5.toInt8()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "maximum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            max(3.toInt8(), 5.toInt8()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "5"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "5"
      End
    End
  End

  Describe "int16 (not default)"
    Describe "addition"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(add(1.toInt16(), 2).getOr(0));
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "subtraction"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(sub(2.toInt16(), 1).getOr(0));
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "1"
      End

      It "runs agc"
        When run test_agc
        The output should eq "1"
      End
    End

    Describe "multiplication"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(mul(2.toInt16(), 1).getOr(0));
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "2"
      End

      It "runs agc"
        When run test_agc
        The output should eq "2"
      End
    End

    Describe "division"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(div(6.toInt16(), 2).getOr(0));
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "modulus"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(mod(6.toInt16(), 4));
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "2"
      End

      It "runs agc"
        When run test_agc
        The output should eq "2"
      End
    End

    Describe "exponentiation"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(pow(6.toInt16(), 2).getOr(0));
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "36"
      End

      It "runs agc"
        When run test_agc
        The output should eq "36"
      End
    End

    Describe "minimum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            min(3.toInt16(), 5.toInt16()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "maximum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            max(3.toInt16(), 5.toInt16()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "5"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "5"
      End
    End
  End

  Describe "int32 (not default)"
    Describe "addition"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            add(1.toInt32(), 2).getOr(0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "subtraction"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            sub(2.toInt32(), 1).getOr(0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "1"
      End

      It "runs agc"
        When run test_agc
        The output should eq "1"
      End
    End

    Describe "multiplication"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            mul(2.toInt32(), 1).getOr(0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "2"
      End

      It "runs agc"
        When run test_agc
        The output should eq "2"
      End
    End

    Describe "division"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            div(6.toInt32(), 2).getOr(0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "modulus"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            mod(6.toInt32(), 4).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "2"
      End

      It "runs agc"
        When run test_agc
        The output should eq "2"
      End
    End

    Describe "exponentiation"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            pow(6.toInt32(), 2).getOr(0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "36"
      End

      It "runs agc"
        When run test_agc
        The output should eq "36"
      End
    End

    Describe "minimum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            min(3.toInt32(), 5.toInt32()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "maximum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            max(3.toInt32(), 5.toInt32()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "5"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "5"
      End
    End
  End

  Describe "int64 (default)"
    Describe "addition"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((1 + 2) || 0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "subtraction"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((2 - 1) || 0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "1"
      End

      It "runs agc"
        When run test_agc
        The output should eq "1"
      End
    End

    Describe "multiplication"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((2 * 1) || 0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "2"
      End

      It "runs agc"
        When run test_agc
        The output should eq "2"
      End
    End

    Describe "division"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((6 / 2) || 0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "modulus"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(6 % 4);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "2"
      End

      It "runs agc"
        When run test_agc
        The output should eq "2"
      End
    End

    Describe "exponentiation"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((6 ** 2) || 0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "36"
      End

      It "runs agc"
        When run test_agc
        The output should eq "36"
      End
    End

    Describe "minimum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            min(3, 5).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "maximum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            max(3.toInt64(), 5.toInt64()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "5"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "5"
      End
    End
  End

  Describe "float32 (not default)"
    Describe "addition"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((toFloat32(1) + 2) || 0.0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "subtraction"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((toFloat32(2) - 1) || 0.0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "1"
      End

      It "runs agc"
        When run test_agc
        The output should eq "1"
      End
    End

    Describe "multiplication"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((toFloat32(2) * 1) || 0.0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "2"
      End

      It "runs agc"
        When run test_agc
        The output should eq "2"
      End
    End

    Describe "division"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((toFloat32(6) / 2) || 0.0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "sqrt"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(sqrt(toFloat32(36)));
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "6"
      End

      It "runs agc"
        When run test_agc
        The output should eq "6"
      End
    End

    Describe "exponentiation"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            print((toFloat32(6) ** 2) || 0.0);
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "36"
      End

      It "runs agc"
        When run test_agc
        The output should eq "36"
      End
    End

    Describe "minimum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            min(3.toFloat32(), 5.toFloat32()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "maximum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            max(3.toFloat32(), 5.toFloat32()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "5"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "5"
      End
    End
  End

  Describe "float64 (default)"
    Describe "addition"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            ((1.0 + 2.0) || 0.0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "subtraction"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            ((2.0 - 1.0) || 0.0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "1"
      End

      It "runs agc"
        When run test_agc
        The output should eq "1"
      End
    End

    Describe "multiplication"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            ((2.0 * 1.0) || 0.0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "2"
      End

      It "runs agc"
        When run test_agc
        The output should eq "2"
      End
    End

    Describe "division"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            ((6.0 / 2.0) || 0.0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "sqrt"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            sqrt(36.0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "6"
      End

      It "runs agc"
        When run test_agc
        The output should eq "6"
      End
    End

    Describe "exponentiation"
      before() {
        lnn_sourceToAll "
          from @std/app import start, print, exit
          on start {
            ((6.0 ** 2.0) || 0.0).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The output should eq "36"
      End

      It "runs agc"
        When run test_agc
        The output should eq "36"
      End
    End

    Describe "minimum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            min(3.toFloat64(), 5.toFloat64()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "maximum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            max(3.toFloat64(), 5.toFloat64()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "5"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "5"
      End
    End
  End

  Describe "grouping"
    before() {
      lnn_sourceToAll "
        from @std/app import start, print, exit
        on start {
          print((2 / (3)) || 0);
          print((3 / (1 + 2)) || 0);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The output should eq "0
1"
    End

    It "runs agc"
      When run test_agc
      The output should eq "0
1"
    End
  End

  Describe "string"
    Describe "minimum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            min(3.toString(), 5.toString()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "3"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "3"
      End
    End

    Describe "maximum"
      before() {
        sourceToTemp "
          from @std/app import start, print, exit
          on start {
            max(3.toString(), 5.toString()).print();
            emit exit 0;
          }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        Pending conditionals
        When run test_js
        The output should eq "5"
      End

      It "runs agc"
        Pending conditionals
        When run test_agc
        The output should eq "5"
      End
    End
  End
End
