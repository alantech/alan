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

  Describe "basic post"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import post

        on start {
          print(post('https://reqbin.com/echo/post/json', '{\"test\":\"test\"}'));
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
      The output should eq "{\"success\":\"true\"}"
    End

    It "runs agc"
      When run test_agc
      The output should eq "{\"success\":\"true\"}"
    End
  End

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


    FETCHOUTPUT="true
200
2
export const comeGetMe = \"You got me!\""

    It "runs js"
      When run test_js
      The output should eq "$FETCHOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$FETCHOUTPUT"
    End
  End

  Describe "Hello World webserver"
    before() {
      sourceToAll "
        from @std/app import start, exit
        from @std/http import connection, listen, body, send, Connection

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

        on start {
          const serverStatus = listen(8080);
          if serverStatus.isErr() {
            emit exit 1;
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
      When run curl -s localhost:8080
      The output should eq "Hello, World!"
    End

    It "runs agc"
      alan run test_$$/temp.agc 1>/dev/null 2>/dev/null &
      PID=$!
      sleep 1
      When run curl -s localhost:8080
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
End