Include build_tools.sh

Describe "Maybe, Result, and Either"
  Describe "Maybe"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn fiver(val: float64) {
          if val.toInt64() == 5 {
            return some(5)
          } else {
            return none()
          }
        }

        on start {
          const maybe5 = fiver(5.5)
          if maybe5.isSome() {
            print(maybe5.getOr(0))
          } else {
            print('what?')
          }

          const maybeNot5 = fiver(4.4)
          if maybeNot5.isNone() {
            print('Correctly received nothing!')
          } else {
            print('uhhh')
          }

          if maybe5.isSome() {
            print(maybe5 || 0)
          } else {
            print('what?')
          }

          if maybeNot5.isNone() {
            print('Correctly received nothing!')
          } else {
            print('uhhh')
          }
          emit exit 0
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
Correctly received nothing!"

    It "runs js"
      When run node temp.js
      The output should eq "$MAYBEOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$MAYBEOUTPUT"
    End
  End

  Describe "Result"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn reciprocal(val: float64) {
          if val == 0.0 {
            return err('Divide by zero error!')
          } else {
            return ok(1.0 / val)
          }
        }

        on start {
          const oneFifth = reciprocal(5.0)
          if oneFifth.isOk() {
            print(oneFifth.getOr(0.0))
          } else {
            print('what?')
          }

          const oneZeroth = reciprocal(0.0)
          if oneZeroth.isErr() {
            const error = oneZeroth.getErr(noerr())
            print(error.msg)
          } else {
            print('uhhh')
          }

          if oneFifth.isOk() {
            print(oneFifth | 0.0)
          } else {
            print('what?')
          }

          if oneZeroth.isErr() {
            print(oneZeroth | 1.2345)
          } else {
            print('uhhh')
          }
          emit exit 0
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
1.2345"

    It "runs js"
      When run node temp.js
      The output should eq "$RESULTOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$RESULTOUTPUT"
    End
  End

  Describe "Either"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const strOrNum = main('string')
          if strOrNum.isMain() {
            print(strOrNum.getMainOr(''))
          } else {
            print('what?')
          }

          const strOrNum2 = alt(2)
          if strOrNum2.isAlt() {
            print(strOrNum2.getAltOr(0))
          } else {
            print('uhhh')
          }
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    EITHEROUTPUT="string
2"

    It "runs js"
      When run node temp.js
      The output should eq "$EITHEROUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$EITHEROUTPUT"
    End
  End
End
