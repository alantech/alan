Include build_tools.sh

Describe "Type detection"
  Describe "basic types and arrays"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start fn {
          print(type 3)
          print(type 3.14)
          print(type (1 + 2))
          print(type 'str')
          print(type true)
          print(type true == 'bool')

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    TYPEOUTPUT="int64
float64
int64
string
bool
true"

    It "runs js"
      When run node temp.js
      The output should eq "$TYPEOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$TYPEOUTPUT"
    End
  End

  Describe "user types and generics"
    before() {
      # TODO: sourceToAll
      sourceToTemp "
        from @std/app import start, print, exit

        type foo<A, B> {
          bar: A
          baz: B
        }

        type foo2 = foo<int64, float64>

        on start fn {
          let a: foo<string, int64>
          let b: foo<int64, bool>
          let c: foo2
          let d: foo<int64, float64>
          print(type a)
          print(type b)
          print(type type a)
          print(type c)
          print(type d)
          print(type c == type d)

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    GENTYPEOUTPUT="foo<string, int64>
foo<int64, bool>
string
foo<int64, float64>
foo<int64, float64>
true"

    It "runs js"
      Pending type-support
      When run node temp.js
      The output should eq "$GENTYPEOUTPUT"
    End

    It "runs agc"
      Pending type-support
      When run alan-runtime run temp.agc
      The output should eq "$GENTYPEOUTPUT"
    End
  End
End
