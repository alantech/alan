Include build_tools.sh

Describe "Closure Functions"
  before() {
    sourceToTemp "
      from @std/app import start, print, exit

      fn closure(): function {
        let num = 0
        return fn (): int64 {
          num = num + 1
          return num
        }
      }

      on start fn (): void {
        const counter1 = closure()
        const counter2 = closure()
        print(counter1())
        print(counter1())
        print(counter2())
        emit exit 0
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  CLOSURERES="1
2
1"

  It "interprets"
    When run alan-interpreter interpret temp.ln
    The output should eq "$CLOSURERES"
  End
End
