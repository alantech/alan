Include build_tools.sh

Describe "Basic Saturating Math"
  Describe "int8 (not default)"
    Describe "addition"
      before() {
        sourceToAll "
          from @std/app import start, exit
          on start { emit exit sadd(toInt8(1), toInt8(2)); }
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
        sourceToAll "
          from @std/app import start, exit
          on start { emit exit ssub(toInt8(2), toInt8(1)); }
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
        sourceToAll "
          from @std/app import start, exit
          on start { emit exit smul(toInt8(2), toInt8(1)); }
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
        sourceToAll "
          from @std/app import start, exit
          on start { emit exit sdiv(toInt8(6), toInt8(0)); }
        "
      }
      BeforeAll before

      after() {
        cleanTemp
      }
      AfterAll after

      It "runs js"
        When run test_js
        The status should eq "127"
      End

      It "runs agc"
        When run test_agc
        The status should eq "127"
      End
    End

    Describe "exponentiation"
      before() {
        sourceToAll "
          from @std/app import start, exit
          on start { emit exit spow(toInt8(6), toInt8(2)); }
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
  End

  Describe "int16 (not default)"
    Describe "addition"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(sadd(toInt16(1), toInt16(2)));
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(ssub(toInt16(2), toInt16(1)));
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(smul(toInt16(2), toInt16(1)));
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(sdiv(toInt16(6), toInt16(2)));
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

    Describe "exponentiation"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(spow(toInt16(6), toInt16(2)));
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
  End

  Describe "int32 (not default)"
    Describe "addition"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            sadd(1.toInt32(), 2.toInt32()).print();
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            ssub(2.toInt32(), 1.toInt32()).print();
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            smul(2.toInt32(), 1.toInt32()).print();
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            sdiv(6.toInt32(), 2.toInt32()).print();
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

    Describe "exponentiation"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            spow(6.toInt32(), 2.toInt32()).print();
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
  End

  Describe "int64 (default)"
    Describe "addition"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(1 +. 2);
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(2 -. 1);
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(2 *. 1);
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(6 /. 2);
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

    Describe "exponentiation"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(6 **. 2);
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
  End

  Describe "float32 (not default)"
    Describe "addition"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(1) +. toFloat32(2));
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(2) -. toFloat32(1));
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(2) *. toFloat32(1));
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(6) /. toFloat32(2));
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

    Describe "exponentiation"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            print(toFloat32(6) **. toFloat32(2));
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
  End

  Describe "float64 (default)"
    Describe "addition"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            (1.0 +. 2.0).print();
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            (2.0 -. 1.0).print();
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            (2.0 *. 1.0).print();
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
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            (6.0 /. 2.0).print();
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

    Describe "exponentiation"
      before() {
        sourceToAll "
          from @std/app import start, print, exit
          on start {
            (6.0 **. 2.0).print();
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
  End
End
