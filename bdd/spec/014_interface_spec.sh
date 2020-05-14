Include build_tools.sh

Describe "Interfaces"
  before() {
    sourceToTemp "
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
  Before before

  after() {
    cleanTemp
  }
  After after

  It "interprets"
    When run alan-interpreter interpret temp.ln
    The output should eq "'Hello, World'
'5'"
  End
End
