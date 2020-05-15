Include build_tools.sh

Describe "Custom events"
  before() {
    sourceToTemp "
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
  Before before

  after() {
    cleanTemp
  }
  After after

  It "interprets"
    When run alan-interpreter interpret temp.ln
    The output should eq "0
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
  End
End
