Include build_tools.sh

Describe "clone"
  before() {
    sourceToAll "
      from @std/app import start, print, exit

      on start {
        let a = 3
        let b = a.clone()
        a = 4
        print(a)
        print(b)
        let c = [1, 2, 3]
        let d = c.clone()
        d.set(0, 2)
        c.map(fn (val: int): string = val.toString()).join(', ').print()
        d.map(fn (val: int): string = val.toString()).join(', ').print()
        emit exit 0
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  CLONEOUTPUT="4
3
1, 2, 3
2, 2, 3"

  It "runs js"
    When run test_js
    The output should eq "${CLONEOUTPUT}"
  End

  It "runs agc"
    When run test_agc
    The output should eq "${CLONEOUTPUT}"
  End
End