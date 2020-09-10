Include build_tools.sh

Describe "Printing"
  Describe "print function"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        on start {
          print('Hello, World')
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
      The output should eq "Hello, World\n"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "Hello, World\n"
    End
  End

  Describe "stdout event"
    before() {
      sourceToAll "
        from @std/app import start, stdout, exit
        on start {
          emit stdout 'Hello, World'
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
      The output should eq "Hello, World"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "Hello, World"
    End
  End
End
