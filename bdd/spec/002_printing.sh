Include build_tools.sh

Describe "Printing"
  Describe "print function"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit
        on start {
          print('Hello, World')
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
      When run alan interpret temp.ln
      The output should eq "Hello, World\n"
    End
  End

  Describe "stdout event"
    before() {
      sourceToTemp "
        from @std/app import start, stdout, exit
        on start {
          emit 'Hello, World'
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
      When run alan interpret temp.ln
      The output should eq "Hello, World"
    End
  End
End
