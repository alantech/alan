Include build_tools.sh

Describe "Functions"
  before() {
    sourceToTemp "
      from @std/app import start, print, exit

      fn foo() {
        print('foo')
      }

      fn bar(str: string, a: int64, b: int64): string {
        return str * a + b.toString()
      }

      fn baz(pre: string, body: string): void {
        print(pre + bar(body, 1, 2))
      }

      fn closure(): function {
        let num = 0
        return fn (): int64 {
          num = num + 1
          return num
        }
      }

      fn double(a: int64) = a * 2

      prefix ## 10 double

      /**
       * It should be possible to write 'doublesum' as:
       *
       * fn doublesum(a: int64, b: int64) = ##a + ##b
       *
       * but the function definitions are all parsed before the first operator mapping is done.
       */
      fn doublesum(a: int64, b: int64) = a.double() + b.double()

      infix #+# 11 doublesum

      on start fn (): void {
        foo()
        'to bar'.bar(2, 3).print()
        '>> '.baz('text here')
        const counter1 = closure()
        const counter2 = closure()
        print(counter1())
        print(counter1())
        print(counter2())
        4.double().print()
        print(##3)
        4.doublesum(1).print()
        print(2 #+# 3)
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
    The output should eq "foo
to barto bar3
>> text here2
1
2
1
8
6
10
10"
  End
End
