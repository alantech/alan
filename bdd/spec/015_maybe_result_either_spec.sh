Include build_tools.sh

Describe "Maybe, Result, and Either"
  Describe "Maybe"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn fiver(val: float64) {
          if val.toInt64() == 5 {
            return some(5);
          } else {
            return none();
          }
        }

        on start {
          const maybe5 = fiver(5.5);
          if maybe5.isSome() {
            print(maybe5.getOr(0));
          } else {
            print('what?');
          }

          const maybeNot5 = fiver(4.4);
          if maybeNot5.isNone() {
            print('Correctly received nothing!');
          } else {
            print('uhhh');
          }

          if maybe5.isSome() {
            print(maybe5 || 0);
          } else {
            print('what?');
          }

          if maybeNot5.isNone() {
            print('Correctly received nothing!');
          } else {
            print('uhhh');
          }

          maybe5.toString().print();
          maybeNot5.toString().print();

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    MAYBEOUTPUT="5
Correctly received nothing!
5
Correctly received nothing!
5
none"

    It "runs js"
      When run test_js
      The output should eq "$MAYBEOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$MAYBEOUTPUT"
    End
  End

  Describe "Result"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn reciprocal(val: float64) {
          if val == 0.0 {
            return err('Divide by zero error!');
          } else {
            return 1.0 / val;
          }
        }

        on start {
          const oneFifth = reciprocal(5.0);
          if oneFifth.isOk() {
            print(oneFifth.getOr(0.0));
          } else {
            print('what?');
          }

          const oneZeroth = reciprocal(0.0);
          if oneZeroth.isErr() {
            const error = oneZeroth.getErr(noerr());
            print(error);
          } else {
            print('uhhh');
          }

          if oneFifth.isOk() {
            print(oneFifth || 0.0);
          } else {
            print('what?');
          }

          if oneZeroth.isErr() {
            print(oneZeroth || 1.2345);
          } else {
            print('uhhh');
          }

          oneFifth.toString().print();
          oneZeroth.toString().print();

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    RESULTOUTPUT="0.2
Divide by zero error!
0.2
1.2345
0.2
Divide by zero error!"

    It "runs js"
      When run test_js
      The output should eq "$RESULTOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$RESULTOUTPUT"
    End
  End

  Describe "Either"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const strOrNum = getMainOrAlt(true);
          if strOrNum.isMain() {
            print(strOrNum.getMainOr(''));
          } else {
            print('what?');
          }

          const strOrNum2 = getMainOrAlt(false);
          if strOrNum2.isAlt() {
            print(strOrNum2.getAltOr(0));
          } else {
            print('uhhh');
          }

          strOrNum.toString().print();
          strOrNum2.toString().print();

          emit exit 0;
        }

        fn getMainOrAlt(isMain: bool) {
          if isMain {
            return main('string');
          } else {
            return alt(2);
          }
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    EITHEROUTPUT="string
2
string
2"

    It "runs js"
      When run test_js
      The output should eq "$EITHEROUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$EITHEROUTPUT"
    End
  End
End
