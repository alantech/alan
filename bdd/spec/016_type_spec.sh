Include build_tools.sh

Describe "Type detection"
  Describe "user types and generics"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        type foo<A, B> {
          bar: A,
          baz: B
        }

        type foo2 = foo<int64, float64>

        on start fn {
          let a = new foo<string, int64> {
            bar: 'bar',
            baz: 0
          };
          let b = new foo<int64, bool> {
            bar: 0,
            baz: true
          };
          let c = new foo2 {
            bar: 0,
            baz: 1.23
          };
          let d = new foo<int64, float64> {
            bar: 1,
            baz: 3.14
          };
          print(a.bar);
          print(b.bar);
          print(c.bar);
          print(d.bar);

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    GENTYPEOUTPUT="bar
0
0
1"

    It "runs js"
      When run test_js
      The output should eq "$GENTYPEOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$GENTYPEOUTPUT"
    End
  End
End
