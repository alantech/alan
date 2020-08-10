Include build_tools.sh

Describe "Arrays"
  Describe "accessor syntax and length"
  before() {
    sourceToAll "
      from @std/app import start, print, exit

      on start {
        print('Testing...')
        const test = '1,2,3'.split(',')
        print(test.length())
        print(test[0])
        print(test[1])
        print(test[2])
        emit exit 0
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  ACCESSOROUTPUT="Testing...
3
1
2
3"

    It "runs js"
      When run node temp.js
      The output should eq "$ACCESSOROUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$ACCESSOROUTPUT"
    End
  End

  Describe "literal syntax"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print('Testing...')
          const test = new Array<int64> [ 1, 2, 3 ]
          print(test[0])
          print(test[1])
          print(test[2])
          const test2 = [ 4, 5, 6 ]
          print(test2[0])
          print(test2[1])
          print(test2[2])
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    LITERALOUTPUT="Testing...
1
2
3
4
5
6"

    It "runs js"
      When run node temp.js
      The output should eq "$LITERALOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$LITERALOUTPUT"
    End
  End

  Describe "push to lazy-let-defined Array and pop from it"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          print('Testing...')
          let test = new Array<int64> []
          test.push(1)
          test.push(2)
          test.push(3)
          print(test[0])
          print(test[1])
          print(test[2])
          print(test.pop())
          print(test.pop())
          print(test.pop())
          print(test.pop()) // Should print error message
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    PUSHOUTPUT="Testing...
1
2
3
3
2
1
cannot pop empty array"

    It "runs js"
      When run node temp.js
      The output should eq "$PUSHOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$PUSHOUTPUT"
    End
  End

  Describe "length, index, has and join"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const test = new Array<int64> [ 1, 1, 2, 3, 5, 8 ]
          const test2 = new Array<string> [ 'Hello', 'World!' ]
          print('has test')
          print(test.has(3))
          print(test.has(4))

          print('length test')
          test.length().print()
          print(#test)

          print('index test')
          test.index(5).print()
          print(test2 @ 'Hello')

          print('join test')
          test2.join(', ').print()

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    LIJARRAYOUTPUT="has test
true
false
length test
6
6
index test
4
0
join test
Hello, World!"

    It "runs js"
      When run node temp.js
      The output should eq "$LIJARRAYOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$LIJARRAYOUTPUT"
    End
  End

  Describe "ternary abuse"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const test = '1':'1':'2':'3':'5':'8'
          print(test.join(', '))

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    TERNARRAYOUTPUT="1, 1, 2, 3, 5, 8"

    It "runs js"
      When run node temp.js
      The output should eq "$TERNARRAYOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$TERNARRAYOUTPUT"
    End
  End

  Describe "map function"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const count = [1, 2, 3, 4, 5] // Ah, ah, ahh!
          const byTwos = count.map(fn (n: int64): int64 = n * 2)
          count.map(fn (n: int64) = toString(n)).join(', ').print()
          byTwos.map(fn (n: int64) = toString(n)).join(', ').print()
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    MAPOUTPUT="1, 2, 3, 4, 5
2, 4, 6, 8, 10"

    It "runs js"
      When run node temp.js
      The output should eq "$MAPOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$MAPOUTPUT"
    End
  End

  Describe "repeat and mapl"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const arr = [1, 2, 3] * 3
          const out = arr.mapl(fn (x: int64): string = x.toString()).join(', ')
          print(out)
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    MAPLOUTPUT="1, 2, 3, 1, 2, 3, 1, 2, 3"

    It "runs js"
      When run node temp.js
      The output should eq "$MAPLOUTPUT"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "$MAPLOUTPUT"
    End
  End

  Describe "each and find"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const test = [ 1, 1, 2, 3, 5, 8 ]
          test.find(fn (val: int64): int64 = val % 2 == 1).getOr(0).print()
          test.each(fn (val: int64) = print('=' * val))
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    EACHFINDOUTPUT="1
=
=
==
===
=====
========"

    agc() {
      export LC_ALL=C
      alan-runtime run temp.agc | sort
    }

    It "runs js"
      When run node temp.js
      The output should eq "$EACHFINDOUTPUT"
    End

    It "runs agc"
      When run agc
      The output should eq "$EACHFINDOUTPUT"
    End
  End

  Describe "everything else ;)"
    before() {
      # TODO: sourceToAll
      sourceToTemp "
        from @std/app import start, print, exit

        fn isOdd(val: int64): bool {
          return val % 2 == 1
        }

        on start {
          print('reduce test')
          test.reduce(add).print()

          print('filter test')
          test.filter(isOdd).each(print)

          print('find test')
          test.find(isOdd).print()

          print('every test')
          test.every(isOdd).print()

          print('some test')
          test.some(isOdd).print()

          print('concat test')
          test.concat(test2).each(print)
          (test + test2).each(print)

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    ADVARRAYOUTPUT="reduce test
20
filter test
1
1
3
5
find test
1
every test
false
some test
true
concat test
1
1
2
3
5
8
2
2
4
6
10
16
1
1
2
3
5
8
2
2
4
6
10
16"

    It "runs js"
      Pending array-support
      When run node temp.js
      The output should eq "$ADVARRAYOUTPUT"
    End

    It "runs agc"
      Pending array-support
      When run alan-runtime run temp.agc
      The output should eq "$ADVARRAYOUTPUT"
    End
  End
End
