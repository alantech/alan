Include build_tools.sh

Describe "Type detection"
  before() {
    sourceToTemp "
      from @std/app import start, print, exit

      type foo<A, B> {
        bar: A
        baz: B
      }

      type foo2 = foo<int64, float64>

      on start fn {
        print(type 3)
        print(type 3.14)
        print(type (1 + 2))
        print(type 'str')
        print(type true)
        print(type true == 'bool')
        
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
  Before before

  after() {
    cleanTemp
  }
  After after

  It "interprets"
    When run alan interpret temp.ln
    The output should eq "int64
float64
int64
string
bool
true
foo<string, int64>
foo<int64, bool>
string
foo<int64, float64>
foo<int64, float64>
true"
  End
End
