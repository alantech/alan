Include build_tools.sh

Describe "Booleans"
  before() {
    sourceToAll "
      from @std/app import start, print, exit

      on start {
        print(true)
        print(false)
        print(toBool(1))
        print(toBool(0))
        print(toBool(15))
        print(toBool(-1))
        print(toBool(0.0))
        print(toBool(1.2))
        print(toBool(''))
        print(toBool('hi'))

        print(true && true)
        print(and(true, false))
        print(false & true)
        print(false.and(false))

        print(true || true)
        print(or(true, false))
        print(false | true)
        print(false.or(false))

        print(true ^ true)
        print(xor(true, false))
        print(false ^ true)
        print(false.xor(false))

        print(!true)
        print(not(false))

        print(true !& true)
        print(nand(true, false))
        print(false !& true)
        false.nand(false).print()

        print(true !| true)
        print(nor(true, false))
        print(false !| true)
        false.nor(false).print()

        print(true !^ true)
        print(xnor(true, false))
        print(false !^ true)
        false.xnor(false).print()

        emit exit 0
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  OUTPUT="true
false
true
false
true
true
false
true
false
false
true
false
false
false
true
true
true
false
false
true
true
false
false
true
false
true
true
true
false
false
false
true
true
false
false
true"

  It "runs js"
    When run node temp.js
    The output should eq "$OUTPUT"
  End

  It "runs agc"
    When run alan-runtime run temp.agc
    The output should eq "$OUTPUT"
  End
End
