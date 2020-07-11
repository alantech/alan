Include build_tools.sh

Describe "Object literals"
  Describe "compiler checks"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        type Foo {
          bar: string
          baz: bool
        }

        on start {
          const foo = new Foo {
            bay = 1.23
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

    It "doesn't work"
      When run alan-compile temp.ln temp.amm
      The status should not eq "0"
      The error should eq "Foo object literal improperly defined
Missing fields: bar, baz
Extra fields: bay
new Foo {
            bay = 1.23
          } on line 2:22"
    End
  End

  Describe "array literals and access"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const test3 = new Array<int64> [ 1, 2, 4, 8, 16, 32, 64 ]
          print(test3[0])
          print(test3[1])
          print(test3[2])

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    ARRTYPEOUTPUT="1
2
4"

    It "runs js"
      When run node temp.js
      The output should eq "$ARRTYPEOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$ARRTYPEOUTPUT"
    End
  End

  Describe "object literals and access"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        type MyType {
          foo: string
          bar: bool
        }

        on start {
          const test = new MyType {
            foo = 'foo!'
            bar = true
          }
          print(test.foo)
          print(test.bar)

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    OBJTYPEOUTPUT="foo!
true"

    It "runs js"
      When run node temp.js
      The output should eq "$OBJTYPEOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$OBJTYPEOUTPUT"
    End
  End

  Describe "object and array reassignment"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        type Foo {
          bar: bool
        }

        on start {
          let test = new Array<int64> [ 1, 2, 3 ]
          print(test[0])
          test[0] = 0
          print(test[0])

          let test2 = new Array<Foo> [
            new Foo {
              bar = true
            },
            new Foo {
              bar = false
            }
          ]
          print(test2[0].bar)
          test2[0].bar = false
          print(test2[0].bar)

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    REASSIGNTYPEOUTPUT="1
0
true
false"

    It "runs js"
      When run node temp.js
      The output should eq "$REASSIGNTYPEOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$REASSIGNTYPEOUTPUT"
    End
  End

  Describe "map support"
    before() {
      # TODO: sourceToAll
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          const test5 = new Map<bool, int64> {
            true: 1
            false: 0
          }

          print(test5[true])
          print(test5[false])

          let test6 = new Map<string, string> {
            'foo': 'bar'
          }
          test6['foo'] = 'baz'
          print(test6['foo'])

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    TYPEOUTPUT="1
0
baz"

    It "runs js"
      Pending type-support
      When run node temp.js
      The output should eq "$TYPEOUTPUT"
    End

    It "runs agc"
      Pending type-support
      When run alan-runtime run temp.agc
      The output should eq "$TYPEOUTPUT"
    End
  End
End
