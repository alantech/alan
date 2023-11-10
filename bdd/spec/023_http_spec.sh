Include build_tools.sh

Describe "@std/http"
  Describe "basic get"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import get

        on start {
          print(get('https://raw.githubusercontent.com/alantech/hellodep/aea1ce817a423d00107577a430a046993e4e6cad/index.ln'));
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The output should eq "export const comeGetMe = \"You got me!\""
    End

    It "runs agc"
      When run test_agc
      The output should eq "export const comeGetMe = \"You got me!\""
    End
  End

# TODO: Revive this test when an alternative to reqbin is found. It no longer works.
#  Describe "basic post"
#    before() {
#      sourceToAll "
#        from @std/app import start, print, exit
#        from @std/http import post
#
#        on start {
#          print(post('https://reqbin.com/echo/post/json', '{\"test\":\"test\"}'));
#          emit exit 0;
#        }
#      "
#    }
#    BeforeAll before
#
#    after() {
#      cleanTemp
#    }
#    AfterAll after
#
#    It "runs js"
#      When run test_js
#      The output should eq "{\"success\":\"true\"}"
#    End
#
#    It "runs agc"
#      When run test_agc
#      The output should eq "{\"success\":\"true\"}"
#    End
#  End

  Describe "fetch directly"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import fetch, Request, Response

        on start {
          const res = fetch(new Request {
            method: 'GET',
            url: 'https://raw.githubusercontent.com/alantech/hellodep/aea1ce817a423d00107577a430a046993e4e6cad/index.ln',
            headers: newHashMap('Content-Length', '0'),
            body: '',
          });
          print(res.isOk());
          const r = res.getOrExit();
          print(r.status);
          print(r.headers.length());
          print(r.body);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    # The number of headers returned in the two runtimes is slightly different. Node includes the
    # "connection: close" header and Hyper.rs does not
    FETCHJSOUTPUT="true
200
25
export const comeGetMe = \"You got me!\""

    FETCHAGCOUTPUT="true
200
23
export const comeGetMe = \"You got me!\""

    It "runs js"
      When run test_js
      The output should eq "$FETCHJSOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$FETCHAGCOUTPUT"
    End
  End

  Describe "Hello World webserver"
    before() {
      sourceToAll "
        from @std/app import start, exit
        from @std/httpserver import connection, body, send, Connection

        on connection fn (conn: Connection) {
          const req = conn.req;
          const res = conn.res;
          set(res.headers, 'Content-Type', 'text/plain');
          if req.method == 'GET' {
            res.body('Hello, World!').send();
          } else {
            res.body('Hello, Failure!').send();
          }
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    afterEach() {
      kill $PID
      wait $PID 2>/dev/null
      return 0
    }
    After afterEach

    It "runs js"
      node test_$$/temp.js 1>/dev/null 2>/dev/null &
      PID=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End

    It "runs agc"
      alan run test_$$/temp.agc 1>/dev/null 2>/dev/null &
      PID=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End
  End

  Describe "importing http get doesn't break hashmap get"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import get

        on start {
          const str = get('https://raw.githubusercontent.com/alantech/hellodep/aea1ce817a423d00107577a430a046993e4e6cad/index.ln').getOr('');
          const kv = str.split(' = ');
          const key = kv[0] || 'bad';
          const val = kv[1] || 'bad';
          const hm = newHashMap(key, val);
          hm.get(key).getOr('failed').print();
          hm.get('something else').getOr('correct').print();
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    GETGETOUTPUT="\"You got me!\"

correct"

    It "runs js"
      When run test_js
      The output should eq "${GETGETOUTPUT}"
    End

    It "runs agc"
      When run test_agc
      The output should eq "${GETGETOUTPUT}"
    End
  End

  Describe "Double-send in a single connection doesn't crash"
    before() {
      sourceToAll "
        from @std/app import print, exit
        from @std/httpserver import connection, Connection, body, send

        on connection fn (conn: Connection) {
          const res = conn.res;
          const firstMessage = res.body('First Message').send();
          print(firstMessage);
          const secondMessage = res.body('Second Message').send();
          print(secondMessage);
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

    It "runs js"
      node test_$$/temp.js 1>./out.txt 2>/dev/null &
      sleep 1
      When run curl -s localhost:8000
      The output should eq "First Message"
    End

    It "response from js"
      When run cat ./out.txt
      The output should eq "HTTP server listening on port 8000
ok
connection not found"
      rm out.txt
    End

    It "runs agc"
      sleep 2
      alan run test_$$/temp.agc 1>./out.txt 2>/dev/null &
      sleep 1
      When run curl -s localhost:8000
      The output should eq "First Message"
    End

    It "response from agc"
      When run cat ./out.txt
      The output should eq "HTTP server listening on port 8000
ok
cannot call send twice for the same connection"
      rm out.txt
    End
  End

End