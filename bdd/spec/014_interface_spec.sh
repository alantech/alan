Include build_tools.sh

Describe "Interfaces"
  before() {
    sourceToAll "
      from @std/app import start, print, exit

      interface Stringifiable {
        toString(Stringifiable): string
      }

      fn quoteAndPrint(toQuote: Stringifiable) {
        print(\"'\" + toString(toQuote) + \"'\")
      }

      on start {
        quoteAndPrint('Hello, World')
        quoteAndPrint(5)
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
    The output should eq "'Hello, World'
'5'"
  End

  It "runs agc"
    When run alan-runtime run temp.agc
    The output should eq "'Hello, World'
'5'"
  End
End
