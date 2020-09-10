Include build_tools.sh

Describe "Bitwise"
  OUTPUT="0
3
6
-1
-1
-4
-7"

  Describe "int8"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        prefix toInt8 as ~ precedence 10

        on start {
          print(~1 & ~2)
          print(~1 | ~3)
          print(~5 ^ ~3)
          print(! ~0)
          print(~1 !& ~2)
          print(~1 !| ~2)
          print(~5 !^ ~3)
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "$OUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$OUTPUT"
    End
  End

  Describe "int16"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        prefix toInt16 as ~ precedence 10

        on start {
          print(~1 & ~2)
          print(~1 | ~3)
          print(~5 ^ ~3)
          print(! ~0)
          print(~1 !& ~2)
          print(~1 !| ~2)
          print(~5 !^ ~3)
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "$OUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$OUTPUT"
    End
  End

  Describe "int32"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        prefix toInt32 as ~ precedence 10

        on start {
          print(~1 & ~2)
          print(~1 | ~3)
          print(~5 ^ ~3)
          print(! ~0)
          print(~1 !& ~2)
          print(~1 !| ~2)
          print(~5 !^ ~3)
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "$OUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$OUTPUT"
    End
  End

  Describe "int64"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print(1 & 2)
          print(1 | 3)
          print(5 ^ 3)
          print(!0)
          print(1 !& 2)
          print(1 !| 2)
          print(5 !^ 3)
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "$OUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$OUTPUT"
    End
  End
End
