Include build_tools.sh

Describe "Strings"
  Describe "most operations"
    before() {
      lnn_sourceToAll "
        from @std/app import start, print, exit

        on start {
          concat('Hello, ', \"World!\").print();
          print('Hello, ' + 'World!');

          repeat('hi ', 5).print();
          print('hey ' * 5);

          matches('foobar', 'fo.*').print();
          print('foobar' ~ 'fo.*');

          // index('foobar', 'ba').print();
          // print('foobar' @ 'ra');

          length('foobar').print();
          print(#'foo');

          trim('   hi  ').print();
          print(\`' hey   ');

          // split('Hello, World!', ', ')[0].print();
          // print(('Hello, World!' / ', ')[1]);

          // const res = split('Hello, World!', ', ');
          // res[0].print();

          // const res2 = 'Hello, World!' / ', ';
          // print(res2[1]);

          wait(1000);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    OUTPUT="Hello, World!
Hello, World!
hi hi hi hi hi 
hey hey hey hey hey 
true
false
6
3
hi
hey
"
    OUTPUT1="Hello, World!"
    OUTPUT2="hi hi hi hi hi"
    OUTPUT3="true"
    OUTPUT4="6"
    OUTPUT5="hi"

    It "runs js"
      When run test_js
      The output should include "$OUTPUT1"
      The output should include "$OUTPUT2"
      The output should include "$OUTPUT3"
      The output should include "$OUTPUT4"
      The output should include "$OUTPUT5"
    End

    It "runs agc"
      When run test_agc
      The output should include "$OUTPUT1"
      The output should include "$OUTPUT2"
      The output should include "$OUTPUT3"
      The output should include "$OUTPUT4"
      The output should include "$OUTPUT5"
    End
  End
End
