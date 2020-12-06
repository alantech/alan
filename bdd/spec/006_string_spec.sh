Include build_tools.sh

Describe "Strings"
  Describe "most operations"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        on start {
          concat('Hello, ', 'World!').print();
          print('Hello, ' + 'World!');

          repeat('hi ', 5).print();
          print('hi ' * 5);

          matches('foobar', 'fo.*').print();
          print('foobar' ~ 'fo.*');

          index('foobar', 'ba').print();
          print('foobar' @ 'ba');

          length('foobar').print();
          print(#'foobar');

          trim('   hi   ').print();
          print(\`'   hi   ');

          split('Hello, World!', ', ')[0].print();
          print(('Hello, World!' / ', ')[1]);

          const res = split('Hello, World!', ', ');
          res[0].print();

          const res2 = 'Hello, World!' / ', ';
          print(res2[1]);

          emit exit 0;
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
Hello
World!"

    It "runs js"
      When run test_js
      The output should eq "$STROUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$STROUTPUT"
    End
  End

  Describe "templating"
    before() {
      # TODO: sourceToAll
      sourceToTemp "
        from @std/app import start, print, exit

        on start {
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
    TMPLOUTPUT="Hello, World!
Good-bye, World!"

    It "runs js"
      Pending template-support
      When run test_js
      The output should eq "$TMPLOUTPUT"
    End

    It "runs agc"
      Pending template-support
      When run test_agc
      The output should eq "$TMPLOUTPUT"
    End
  End
End
