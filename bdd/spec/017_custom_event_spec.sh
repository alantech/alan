Include build_tools.sh

Describe "Custom events"
  OUTPUT="0
1
2
3
4
5
6
7
8
9
10"

  Describe "normal exit code"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        event loop: int64

        on loop fn looper(val: int64) {
          print(val)
          if val >= 10 {
            emit exit 0
          } else {
            emit loop val + 1
          }
        }

        on start {
          emit loop 0
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
      The status should eq "$OUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The status should eq "$OUTPUT"
    End
  End
End
