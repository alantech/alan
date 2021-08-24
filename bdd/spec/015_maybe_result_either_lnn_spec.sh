Include build_tools.sh

Describe "Maybe, Result, and Either"
  Describe "basic working check"
    before() {
      lnn_sourceToAll "
        from @std/app import start, print, exit

        on start {
          let result = ok(4);
          result.getOr(0).print();
          result = err(noerr());
          result.getOr(0).print();
          result = ok(0);
          result.getOr(3).print();

          let maybe = some(4);
          maybe.getOr(0).print();
          maybe = none();
          maybe.getOr(0).print();
          maybe = some(0);
          maybe.getOr(3).print();

          let either = main(4);
          either.getMainOr(0).print();
          either = alt('hello world');
          either.getAltOr('').print();
          either = main(0);
          either.getMainOr(3).print();
          either = alt('');
          either.getAltOr('hello world').print();

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    OUTPUT="4
0
3
4
0
3
4
hello world
0

"

    It "runs js"
      When run test_js
      The output should eq "$OUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$OUTPUT"
    End
  End

  Describe "Maybe"
    before() {
      lnn_sourceToTemp "
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
      Pending conditionals
      When run test_js
      The output should eq "$MAYBEOUTPUT"
    End

    It "runs agc"
      Pending conditionals
      When run test_agc
      The output should eq "$MAYBEOUTPUT"
    End
  End

  Describe "Result"
    before() {
      lnn_sourceToTemp "
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

          const res = ok('foo');
          print(res.getErr('there is no error'));

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
Divide by zero error!
there is no error"

    It "runs js"
      Pending conditionals
      When run test_js
      The output should eq "$RESULTOUTPUT"
    End

    It "runs agc"
      Pending conditionals
      When run test_agc
      The output should eq "$RESULTOUTPUT"
    End
  End

  Describe "Either"
    before() {
      lnn_sourceToTemp "
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
      Pending conditionals
      When run test_js
      The output should eq "$EITHEROUTPUT"
    End

    It "runs agc"
      Pending conditionals
      When run test_agc
      The output should eq "$EITHEROUTPUT"
    End
  End
End
