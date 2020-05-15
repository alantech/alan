Include build_tools.sh

Describe "Unions"
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
        // needs safeguards. The interpreter can just roll with this, but not sure how to compile-time
        // validate this, yet.
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
  Before before

  after() {
    cleanTemp
  }
  After after

  It "interprets"
    When run alan-interpreter interpret temp.ln
    The output should eq "safeFraction
0.5
2.0
0.0
Infinity
safeFraction2
0.5
2.0
0.0
Infinity"
  End
End
