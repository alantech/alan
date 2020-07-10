Include build_tools.sh

Describe "Maybe, Result, and Either"
  Describe "Maybe"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        // TODO: Return type inference from conditional functions
        fn fiver(val: float64): Maybe<int64> {
          if val.toInt64() == 5 {
            return some(5)
          } else {
            return none()
          }
        }

        on start {
          const maybe5 = fiver(5.5)
          if maybe5.isSome() {
            print(maybe5.get(0))
          } else {
            print('what?')
          }
          
          const maybeNot5 = fiver(4.4)
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

        // TODO: Return type inference from conditional functions
        fn reciprocal(val: float64): Result<float64> {
          if val == 0.0 {
            return err('Divide by zero error!')
          } else {
            return ok(1.0 / val)
          }
        }

        on start {
          const oneFifth = reciprocal(5.0)
          if oneFifth.isOk() {
            print(oneFifth.get(0))
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

  Describe "Unions (to be deleted)"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        fn reciprocal(val: float64): float64 | Error {
          if val == 0.0 {
            return new Error {
              msg = 'Divide by zero error!'
              code = 0
            }
          }
          return 1.0 / val
        }

        fn safeFraction(numerator: float64, denominator: float64): float64 {
          const recipDenom = reciprocal(denominator)
          if type recipDenom == 'Error' { // TODO: Is the string comparison approach the right one?
            if recipDenom.code == 0 {
              return toFloat64('Infinity') // TODO: Missing some constants for floating point math
            } else {
              return toFloat64('NaN')
            }
          } else if type recipDenom == 'float64' { // TODO: How to enforce all union types are handled?
            return numerator * recipDenom
          } else {
            // TODO: This block should not be necessary, but it will be hard to detect that the else block
            // handles the remaining union type (maybe just 'suck it up' is the answer?)
            return 0.0
          }
        }

        fn safeFraction2(numerator: float64, recipDenom: float64 | Error): float64 {
          if type recipDenom == 'Error' {
            if recipDenom.code == 0 {
              return toFloat64('Infinity')
            } else {
              return toFloat64('NaN')
            }
          }
          // TODO: This is another example where, when the type checking is down to just one, it no longer
          // needs safeguards, but not sure how to compile-time validate this, yet.
          return numerator * recipDenom
        }

        on start {
          print('safeFraction')
          print(safeFraction(1.0, 2.0))
          print(safeFraction(2.0, 1.0))
          print(safeFraction(0.0, 2.0))
          print(safeFraction(2.0, 0.0))

          print('safeFraction2')
          print(safeFraction2(1.0, reciprocal(2.0)))
          print(safeFraction2(2.0, reciprocal(1.0)))
          print(safeFraction2(0.0, reciprocal(2.0)))
          print(safeFraction2(2.0, reciprocal(0.0)))

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    UNIONOUTPUT="safeFraction
0.5
2.0
0.0
Infinity
safeFraction2
0.5
2.0
0.0
Infinity"

    It "runs js"
      Pending union-support
      When run node temp.js
      The output should eq "$UNIONOUTPUT"
    End

    It "runs agc"
      Pending union-support
      When run alan-runtime run temp.agc
      The output should eq "$UNIONOUTPUT"
    End
  End
End
