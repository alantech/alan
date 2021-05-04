Include build_tools.sh

Describe "Strings"
  Describe "most operations"
    before() {
      lnn_sourceToAll "
        from @std/app import start, stdout, exit

        on start {
          emit stdout concat('Hello, ', 'World!\n');
          // concat('Hello, ', 'World!').print();
          // print('Hello, ' + 'World!');

          emit stdout concat(repeat('hi ', 5), '\n');
          // repeat('hi ', 5).print();
          // print('hi ' * 5);

          emit stdout concat(toString(matches('foobar', 'fo.*')), '\n');
          // matches('foobar', 'fo.*').print();
          // print('foobar' ~ 'fo.*');

          emit stdout concat(toString(index('foobar', 'ba')), '\n');
          // index('foobar', 'ba').print();
          // print('foobar' @ 'ba');

          emit stdout concat(toString(length('foobar')), '\n');
          // length('foobar').print();
          // print(#'foobar');

          emit stdout concat(trim('   hi   '), '\n');
          // trim('   hi   ').print();
          // print(\`'   hi   ');

          // split('Hello, World!', ', ')[0].print();
          // print(('Hello, World!' / ', ')[1]);

          // const res = split('Hello, World!', ', ');
          // res[0].print();

          // const res2 = 'Hello, World!' / ', ';
          // print(res2[1]);

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
hi hi hi hi hi 
true
3
6
hi
"

    It "runs js"
      When run test_js
      The output should eq "$STROUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$STROUTPUT"
    End
  End
End
