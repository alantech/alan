Include build_tools.sh

Describe "Printing"
  Describe "print function"
    before() {
      lnn_sourceToAll "
        from @std/app import start, print, exit
        on start {
          print('Hello, World');
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The output should eq "Hello, World"
    End

    It "runs agc"
      When run test_agc
      The output should eq "Hello, World"
    End
  End

  Describe "stdout event"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit
        on start {
          emit stdout 'Hello, World';
          wait(10);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The output should eq "Hello, World"
    End

    It "runs agc"
      When run test_agc
      The output should eq "Hello, World"
    End
  End
End
