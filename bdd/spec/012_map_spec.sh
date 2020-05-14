Include build_tools.sh

Describe "Maps"
  before() {
    sourceToTemp "
      from @std/app import start, print, exit

      on start {
        const test = new Map<string, int64> {
          'foo': 1
          'bar': 2
          'baz': 99
        }

        print('keyVal test')
        test.keyVal().each(fn (n: KeyVal<string, int64>) {
          print('key: ' + n.key)
          print('val: ' + n.value.toString())
        })

        print('keys test')
        test.keys().each(print)

        print('values test')
        test.values().each(print)

        print('length test')
        test.length().print()
        print(#test)

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
    The output should eq "keyVal test
key: bar
val: 2
key: foo
val: 1
key: baz
val: 99
keys test
bar
foo
baz
values test
2
1
99
length test
3
3"
  End
End
