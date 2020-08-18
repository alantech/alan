Include build_tools.sh

Describe "Hashing"
  Describe "toHash"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print(toHash(1))
          print(toHash(3.14159))
          print(toHash(true))
          print(toHash('false'))
          print(toHash([1, 2, 5, 3]))
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    TOHASHOUTPUT="-1058942856030168491
-5016367128657347516
-1058942856030168491
6288867289231076425
-1521185239552941064"

    JSHASHOUTPUT="-1058942856030168400
-5016367128657348000
-1058942856030168400
6288867289231076000
-1521185239552941000" # TODO: Rounding should disappear once we can use BigInt consistently in JS

    It "runs js"
      When run node temp.js
      The output should eq "$JSHASHOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$TOHASHOUTPUT"
    End
  End

  Describe "HashMap (no syntactic sugar)"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const test = newHashMap('foo', 1)
          test.set('bar', 2)
          test.set('baz', 99)
          print(test.keyVal().map(fn (n: KeyVal<string, int64>): string {
            return 'key: ' + n.key + \"\\nval: \" + toString(n.val)
          }).join(\"\\n\"))
          print(test.keys().join(', '))
          print(test.vals().map(fn (n: int64): string = n.toString()).join(', '))
          print(test.length())
          print(test.get('foo'))
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    HASHMAPOUTPUT="key: foo
val: 1
key: bar
val: 2
key: baz
val: 99
foo, bar, baz
1, 2, 99
3
1"

    It "runs js"
      When run node temp.js
      The output should eq "$HASHMAPOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$HASHMAPOUTPUT"
    End
  End

  Describe "KeyVal to HashMap"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        fn kv(k: any, v: anythingElse) = new KeyVal<any, anythingElse> {
          key = k
          val = v
        }

        on start {
          const kva = [ kv(1, 'foo'), kv(2, 'bar'), kv(3, 'baz') ]
          const hm = kva.asHashMap()
          print(hm.keyVal().map(fn (n: KeyVal<int64, string>): string {
            return 'key: ' + toString(n.key) + \"\\nval: \" + n.val
          }).join(\"\\n\"))
          print(hm.get(1))
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    KVTOHMOUTPUT="key: 1
val: foo
key: 2
val: bar
key: 3
val: baz
foo"

    It "runs js"
      When run node temp.js
      The output should eq "$KVTOHMOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$KVTOHMOUTPUT"
    End
  End

  Describe "HashMap"
    before() {
      # TODO: sourceToAll
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
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    MAPOUTPUT="keyVal test
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

    It "runs js"
      Pending map-support
      When run node temp.js
      The output should eq "$MAPOUTPUT"
    End

    It "runs agc"
      Pending map-support
      When run alan-runtime run temp.agc
      The output should eq "$MAPOUTPUT"
    End
  End
End
