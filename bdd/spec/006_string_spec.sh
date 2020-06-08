Include build_tools.sh

Describe "Strings"
  before() {
    # TODO: sourceToAll
    sourceToTemp "
      from @std/app import start, print, exit

      on start {
        concat('Hello, ', 'World!').print()
        print('Hello, ' + 'World!')

        repeat('hi ', 5).print()
        print('hi ' * 5)

        matches('foobar', 'fo.*').print()
        print('foobar' ~ 'fo.*')

        index('foobar', 'ba').print()
        print('foobar' @ 'ba')

        length('foobar').print()
        print(#'foobar')

        trim('   hi   ').print()
        print(\`'   hi   ')

        /**
         * The following should work but the grammar doesn't yet support array access without a
         * variable name, so I have to write it in a not-great form :(
         *
         * split('Hello, World!', ', ')[0].print()
         * print(('Hello, World!' / ', ')[1])
         */

        const res = split('Hello, World!', ', ')
        /**
         * You also can't chain off of an array access for some reason.
         *
         * res[0].print()
         */
        print(res[0])

        const res2 = 'Hello, World!' / ', '
        print(res[1])

        template('\${greet}, \${name}!', new Map<string, string> {
          'greet': 'Hello'
          'name': 'World'
        }).print()
        print('\${greet}, \${name}!' % new Map<string, string> {
          'greet': 'Good-bye'
          'name': 'World'
        })

        emit exit 0
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  STROUTPUT="Hello, World!
Hello, World!
hi hi hi hi hi 
hi hi hi hi hi 
true
true
3
3
6
6
hi
hi
Hello
World!
Hello, World!
Good-bye, World!"

  It "runs js"
    Pending string-support
    When run node temp.js
    The output should eq "$STROUTPUT"
  End

  It "runs agc"
    Pending string-support
    When run alan-runtime run temp.agc
    The output should eq "$STROUTPUT"
  End
End
