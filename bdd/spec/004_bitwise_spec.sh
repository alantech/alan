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

        prefix ~ 10 toInt8

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
      When run alan-runtime run temp.agc
      The output should eq "$OUTPUT"
    End
  End

  Describe "int16"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        prefix ~ 10 toInt16

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
      When run alan-runtime run temp.agc
      The output should eq "$OUTPUT"
    End
  End

  Describe "int32"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        prefix ~ 10 toInt32

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
      When run alan-runtime run temp.agc
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
      When run alan-runtime run temp.agc
      The output should eq "$OUTPUT"
    End
  End
End
