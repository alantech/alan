Include build_tools.sh

Describe "@std/http"
  Describe "basic get"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import get

        on start {
          print(get('https://raw.githubusercontent.com/alantech/hellodep/aea1ce817a423d00107577a430a046993e4e6cad/index.ln'))
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "export const comeGetMe = \"You got me!\""
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "export const comeGetMe = \"You got me!\""
    End
  End

  Describe "basic post"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import post

        on start {
          print(post('https://reqbin.com/echo/post/json', '{\"test\":\"test\"}'))
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "{\"success\":\"true\"}"
    End

    It "runs agc"
      When run alan-runtime run temp.agc
      The output should eq "{\"success\":\"true\"}"
    End
  End

  Describe "Hello World webserver"
    before() {
      sourceToAll "
        from @std/app import start, exit
        from @std/http import connection, listen, body, send, Connection

        on connection fn (conn: Connection) {
          const res = conn.res
          set(res.headers, 'Content-Type', 'text/plain')
          const sendStatus = res.body('Hello, World!').send()
        }

        on start {
          const serverStatus = listen(8080)
          if serverStatus.isErr() {
            emit exit 1
          }
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      node temp.js &
      sleep 1
      When run curl -s localhost:8080
      The output should eq "Hello, World!"
      killall node
    End

    It "runs agc"
      Pending rust-webserver
      alan-runtime run temp.agc &
      sleep 1
      When run curl -s localhost:8080
      The output should eq "Hello, World!"
    End
  End
End