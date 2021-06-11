Include build_tools.sh

Describe "@std/tcp"
  Describe "webserver tunnel test"
    before() {
      sourceToTemp "
        from @std/tcpserver import tcpConn
        from @std/tcp import TcpChannel, connect, addContext, ready, chunk, TcpContext, read, write, tcpClose, close

        on tcpConn fn (channel: TcpChannel) {
          const tunnel = connect('localhost', 8088);
          channel.addContext(tunnel);
          tunnel.addContext(channel);
          channel.ready();
          tunnel.ready();
        }

        on chunk fn (ctx: TcpContext<TcpChannel>) {
          ctx.context.write(ctx.channel.read());
        }

        on tcpClose fn (ctx: TcpContext<TcpChannel>) {
          ctx.context.close();
        }
      "
      tempToAmm
      tempToJs
      sourceToFile test_server.js "
        const http = require('http')

        http.createServer((req, res) => res.end('Hello, World!')).listen(8088)
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
      kill $PID2
      wait $PID2 2>/dev/null
      return 0
    }
    After afterEach

    It "runs js"
      node test_$$/test_server.js 1>/dev/null 2>/dev/null &
      PID1=$!
      node test_$$/temp.js 1>/dev/null 2>/dev/null &
      PID2=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End
  End

  Describe "webserver tunnel function test"
    before() {
      sourceToAll "
        from @std/tcpserver import tunnel
        from @std/app import start, print

        on start {
          let connected = tunnel(8088);
          print(connected ? 'Tunneling to 8088' : 'Failed to establish a tunnel');
        }
      "
      sourceToFile test_server.js "
        const http = require('http')

        http.createServer((req, res) => res.end('Hello, World!')).listen(8088)
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
      kill $PID2
      wait $PID2 2>/dev/null
      return 0
    }
    After afterEach

    It "runs js"
      node test_$$/test_server.js 1>/dev/null 2>/dev/null &
      PID1=$!
      node test_$$/temp.js 1>/dev/null 2>/dev/null &
      PID2=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End

    It "runs agc"
      node test_$$/test_server.js 1>/dev/null 2>/dev/null &
      PID1=$!
      alan run test_$$/temp.agc 1>/dev/null 2>/dev/null &
      PID2=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End
  End
End