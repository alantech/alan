Include build_tools.sh

Describe "Closure Functions"
  Describe "closure creation and usage"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn closure(): function {
          let num = 0;
          return fn (): int64 {
            num = num + 1 || 0;
            return num;
          };
        }

        on start fn (): void {
          const counter1 = closure();
          const counter2 = closure();
          print(counter1());
          print(counter1());
          print(counter2());
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    CLOSURERES="1
2
1"

    It "runs js"
      When run test_js
      The output should eq "$CLOSURERES"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$CLOSURERES"
    End
  End

  Describe "closure usage by name"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn double(x: int64): int64 = x * 2 || 0;

        on start {
          const numbers = [1, 2, 3, 4, 5];
          numbers.map(double).map(toString).join(', ').print();
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    REFOUTPUT="2, 4, 6, 8, 10"

    It "runs js"
      When run test_js
      The output should eq "$REFOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$REFOUTPUT"
    End
  End

  Describe "inlined closures with argument"
    before() {
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
          const arghFn = fn(argh: string) {
            print(argh);
          };
          arghFn('argh');
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    OUTPUT="argh"

    It "runs js"
      Pending arguments-for-inlined-closures
      When run test_js
      The output should eq "$OUTPUT"
    End

    It "runs agc"
      Pending arguments-for-inlined-closures
      When run test_agc
      The output should eq "$OUTPUT"
    End
  End
End
