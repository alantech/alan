Include build_tools.sh

Describe "Object literals"
  before() {
    sourceToTemp "
      from @std/app import start, print, exit

      type MyType {
        foo: string
        bar: bool
      }

      on start {
        print('Custom type assignment')
        const test = new MyType {
          foo = 'foo!'
          bar = true
        }
        print(test.foo)
        print(test.bar)

        let test2 = new MyType {
          foo = 'foo2'
          bar = true
        }
        test2.bar = false
        print(test2.foo)
        print(test2.bar)

        print('Array literal assignment')
        const test3 = new Array<int64> [ 1, 2, 4, 8, 16, 32, 64 ]
        print(test3[0])
        print(test3[1])
        print(test3[2])

        let test4 = new Array<int64> [ 0, 1, 2, 3 ]
        test4[0] = 1
        print(test4[0])

        print('Map literal assignment')
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
  Before before

  after() {
    cleanTemp
  }
  After after

  It "interprets"
    When run alan-interpreter interpret temp.ln
    The output should eq "Custom type assignment
foo!
true
foo2
false
Array literal assignment
1
2
4
1
Map literal assignment
1
0
baz"
  End
End
