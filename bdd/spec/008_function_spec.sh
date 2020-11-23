Include build_tools.sh

Describe "Functions and Custom Operators"
  before() {
    sourceToAll "
      from @std/app import start, print, exit

      fn foo() {
        print('foo');
      }

      fn bar(str: string, a: int64, b: int64): string {
        return str * a + b.toString();
      }

      fn baz(pre: string, body: string): void {
        print(pre + bar(body, 1, 2));
      }

      // 'int' is an alias for 'int64'
      fn double(a: int) = a * 2

      prefix double as ## precedence 10

      /**
       * It should be possible to write 'doublesum' as:
       *
       * fn doublesum(a: int64, b: int64) = ##a + ##b
       *
       * but the function definitions are all parsed before the first operator mapping is done.
       */
      fn doublesum(x: int64, y: int64) = x.double() + y.double() // TODO: Fix naming confusion

      infix doublesum as #+# precedence 11

      on start fn (): void {
        foo();
        'to bar'.bar(2, 3).print();
        '>> '.baz('text here');
        4.double().print();
        print(##3);
        4.doublesum(1).print();
        print(2 #+# 3);
        emit exit 0;
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  FUNCTIONRES="foo
to barto bar3
>> text here2
8
6
10
10"

  It "runs js"
    When run test_js
    The output should eq "$FUNCTIONRES"
  End

  It "runs agc"
    When run test_agc
    The output should eq "$FUNCTIONRES"
  End
End
