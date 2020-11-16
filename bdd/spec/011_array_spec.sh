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
      When run alan run temp.agc
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
      When run alan run temp.agc
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
      When run alan run temp.agc
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
      When run alan run temp.agc
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
      When run alan run temp.agc
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
      When run alan run temp.agc
      The output should eq "$MAPOUTPUT"
    End
  End

  Describe "repeat and mapLin"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const arr = [1, 2, 3] * 3
          const out = arr.mapLin(fn (x: int64): string = x.toString()).join(', ')
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
      When run alan run temp.agc
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
      alan run temp.agc | sort
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

  Describe "every and some"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const test = [ 1, 1, 2, 3, 5, 8 ]
          // TODO: Get non-inline closure functions working
          test.every(fn (val: int64): bool = val % 2 == 1).print()
          test.some(fn (val: int64): bool = val % 2 == 1).print()

          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    EVERYSOMEOUTPUT="false
true"

    It "runs js"
      When run node temp.js
      The output should eq "$EVERYSOMEOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$EVERYSOMEOUTPUT"
    End
  End

  Describe "reduce, filter, and concat"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          const test = [ 1, 1, 2, 3, 5, 8 ]
          const test2 = [ 4, 5, 6 ]
          print('reduce test')
          test.reduce(fn (a: int, b: int): int = a + b).print()
          test.reduce(min).print()
          test.reduce(max).print()

          print('filter test')
          test.filter(fn (val: int64): bool {
            return val % 2 == 1
          }).map(fn (val: int64): string {
            return toString(val)
          }).join(', ').print()

          print('concat test')
          test.concat(test2).map(fn (val: int64): string {
            return toString(val)
          }).join(', ').print()
          (test + test2).map(fn (val: int64): string {
            return toString(val)
          }).join(', ').print()

          print('reduce as filter and concat test')
          // TODO: Lots of improvements needed for closures passed directly to opcodes. This one-liner is ridiculous
          test.reduce(fn (acc: string, i: int): string = ((acc == '') && (i % 2 == 1)) ? i.toString() : (i % 2 == 1 ? (acc + ', ' + i.toString()) : acc), '').print()
          // TODO: Even more ridiculous when you want to allow parallelism
          test.reducePar(fn (acc: string, i: int): string = ((acc == '') && (i % 2 == 1)) ? i.toString() : (i % 2 == 1 ? (acc + ', ' + i.toString()) : acc), fn (acc: string, cur: string): string = ((acc != '') && (cur != '')) ? (acc + ', ' + cur) : (acc != '' ? acc : cur), '').print()

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
1
8
filter test
1, 1, 3, 5
concat test
1, 1, 2, 3, 5, 8, 4, 5, 6
1, 1, 2, 3, 5, 8, 4, 5, 6
reduce as filter and concat test
1, 1, 3, 5
1, 1, 3, 5"

    It "runs js"
      When run node temp.js
      The output should eq "$ADVARRAYOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$ADVARRAYOUTPUT"
    End
  End

  Describe "user-defined types in array methods work"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        type Foo {
          foo: string
          bar: bool
        }

        on start {
          const five = [1, 2, 3, 4, 5]
          five.map(fn (n: int64): Foo {
            return new Foo {
              foo = n.toString()
              bar = n % 2 == 0
            }
          }).filter(fn (f: Foo): bool = f.bar).map(fn (f: Foo): string = f.foo).join(', ').print()
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    USERTYPEOUTPUT="2, 4"

    It "runs js"
      When run node temp.js
      The output should eq "$USERTYPEOUTPUT"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "$USERTYPEOUTPUT"
    End
  End
End
