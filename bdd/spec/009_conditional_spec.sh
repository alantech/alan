Include build_tools.sh

Describe "Conditionals"
  Describe "Basic"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn bar() {
          print('bar!');
        }

        fn baz() {
          print('baz!');
        }

        on start {
          if 1 == 0 {
            print('What!?');
          } else {
            print('Math is sane...');
          }

          if 1 == 0 {
            print('Not this again...');
          } else if 1 == 2 {
            print('Still wrong...');
          } else {
            print('Math is still sane, for now...');
          }

          const foo: bool = true == true;
          if foo bar else baz

          const isTrue = true == true;
          cond(isTrue, fn {
            print(\"It's true!\");
          })
          cond(!isTrue, fn {
            print('This should not have run');
          })

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
      The output should eq "Math is sane...
Math is still sane, for now...
bar!
It's true!"
    End

    It "runs agc"
      When run test_agc
      The output should eq "Math is sane...
Math is still sane, for now...
bar!
It's true!"
    End
  End

  Describe "Nested"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          if true {
            print(1);
            if 1 == 2 {
              print('What?');
            } else {
              print(2);
              if 2 == 1 {
                print('Uhh...');
              } else if 2 == 2 {
                print(3);
              } else {
                print('Nope');
              }
            }
          } else {
            print('Hmm');
          }
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
      The output should eq "1
2
3"
    End

    It "runs agc"
      When run test_agc
      The output should eq "1
2
3"
    End
  End

  Describe "Early Return"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn nearOrFar(distance: float64): string {
          if distance < 5.0 {
            return 'Near!';
          } else {
            return 'Far!';
          }
        }

        on start {
          print(nearOrFar(3.14));
          print(nearOrFar(6.28));

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    RETOUTPUT="Near!
Far!"

    It "runs js"
      When run test_js
      The output should eq "$RETOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$RETOUTPUT"
    End
  End

  Describe "Ternary"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const options = pair(2, 4);
          print(options[0]);
          print(options[1]);

          const options2 = 3 : 5;
          print(options2[0]);
          print(options2[1]);

          const val1 = 1 == 1 ? 1 : 2;
          const val2 = 1 == 0 ? 1 : 2;
          print(val1);
          print(val2);

          const val3 = cond(1 == 1, pair(3, 4));
          const val4 = cond(1 == 0, pair(3, 4));
          print(val3);
          print(val4);

          const val5 = 1 == 0 ? options2;
          print(val5);

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    ADVOUTPUT="2
4
3
5
1
2
3
4
5"

    It "runs js"
      When run test_js
      The output should eq "$ADVOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$ADVOUTPUT"
    End
  End
End