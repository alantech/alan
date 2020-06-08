Include build_tools.sh

Describe "Arrays"
  before() {
    # TODO: sourceToAll
    sourceToTemp "
      from @std/app import start, print, exit

      fn isOdd(val: int64): bool {
        return val % 2 == 1
      }

      on start {
        print('each test')
        const test = new Array<int64> [ 1, 1, 2, 3, 5, 8 ]
        test.each(fn (val: int64) = print('=' * val))

        print('map test')
        const test2 = test.map(fn (val: int64) = val * 2)
        test2.each(print)

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

        print('length test')
        test.length().print()
        print(#test)

        print('index test')
        test.index(5).print()
        print(test @ 5)

        print('repeat test')
        (new Array<int64> [0]).repeat(3).each(print)
        each(test * 3, print)

        print('concat test')
        test.concat(test2).each(print)
        (test + test2).each(print)

        print('join test')
        test.map(toString).join(', ').print()

        emit exit 0
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  ARRAYOUTPUT="each test
=
=
==
===
=====
========
map test
2
2
4
6
10
16
reduce test
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
length test
6
6
index test
4
4
repeat test
0
0
0
1
1
2
3
5
8
1
1
2
3
5
8
1
1
2
3
5
8
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
16
join test
1, 1, 2, 3, 5, 8"

  It "runs js"
    Pending array-support
    When run node temp.js
    The output should eq "$ARRAYOUTPUT"
  End

  It "runs agc"
    Pending array-support
    When run alan-runtime run temp.agc
    The output should eq "$ARRAYOUTPUT"
  End
End
