Include build_tools.sh

Describe "Types"
  Describe "user types and generics"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        type foo<A, B> {
          bar: A,
          baz: B
        }

        type foo2 = foo<int64, float64>

        on start fn {
          let a = new foo<string, int64> {
            bar: 'bar',
            baz: 0
          };
          let b = new foo<int64, bool> {
            bar: 0,
            baz: true
          };
          let c = new foo2 {
            bar: 0,
            baz: 1.23
          };
          let d = new foo<int64, float64> {
            bar: 1,
            baz: 3.14
          };
          print(a.bar);
          print(b.bar);
          print(c.bar);
          print(d.bar);

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    GENTYPEOUTPUT="bar
0
0
1"

    It "runs js"
      When run test_js
      The output should eq "$GENTYPEOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$GENTYPEOUTPUT"
    End
  End

  Describe "using non-imported type returned by imported function"
    before() {
      sourceToTemp "
        from @std/app import start, exit
        from @std/http import fetch, Request

        on start {
          arghFn('{\"test\":\"test\"}');
          emit exit 0;
        }

        fn arghFn(arghStr: string) {
          fetch(new Request {
              method: 'POST',
              url: 'https://reqbin.com/echo/post/json',
              headers: newHashMap('Content-Length', arghStr.length().toString()),
              body: arghStr,
            });
        }
      "
      sourceToFile test_server.js "
        const http = require('http');

        http.createServer((req, res) => {
          console.log('received');
          res.end('Hello, world!');
        }).listen(8088);
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    afterEach() {
      kill $PID1
      wait $PID1 2>/dev/null
      # kill $PID2
      # wait $PID2 2>/dev/null
      return 0
    }
    After afterEach

    It "runs js"
      Pending unimported-types-returned-by-imported-functions
      node test_$$/test_server.js 1>test_$$/test_server.js.out 2>/dev/null &
      PID1=$!
      # node test_$$/temp.js 1>/dev/null &
      # PID2=$!
      sleep 1
      When run cat test_$$/test_server.js.out
      The output should eq "received"
    End

    It "runs agc"
      Pending unimported-types-returned-by-imported-functions
      node test_$$/test_server.js 1>test_$$/test_server.agc.out 2>/dev/null &
      PID1=$!
      # alan run test_$$/temp.agc 1>/dev/null 2>/dev/null  &
      # PID2=$!
      sleep 1
      When run cat test_$$/test_server.agc.out
      The output should eq "received"
    End
  End
End
