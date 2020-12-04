Include build_tools.sh

Describe "Module-level Constants"
  Describe "simple constant"
    before() {
      sourceToAll "
        import @std/app

        const helloWorld = 'Hello, World!';

        on app.start {
          app.print(helloWorld);
          emit app.exit 0;
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
      The output should eq "Hello, World!"
    End

    It "runs agc"
      When run test_agc
      The output should eq "Hello, World!"
    End
  End

  Describe "function-defined constants"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        const three = add(1, 2);

        fn fiver() = 5

        const five = fiver();

        on start {
          print(three);
          print(five);
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
      The output should eq "3
5"
    End

    It "runs agc"
      When run test_agc
      The output should eq "3
5"
    End
  End
End