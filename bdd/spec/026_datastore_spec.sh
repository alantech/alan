Include build_tools.sh

Describe "@std/datastore"
  before() {
    sourceToAll "
      from @std/app import start, print, exit
      from @std/datastore import namespace, has, set, del, getOr

      on start {
        const ns = namespace('foo')
        print(ns.has('bar'))
        ns.set('bar', 'baz')
        print(ns.has('bar'))
        print(ns.getOr('bar', ''))
        ns.del('bar')
        print(ns.has('bar'))
        print(ns.getOr('bar', ''))

        ns.set('inc', 0)
        emit waitAndInc 100
        emit waitAndInc 200
        emit waitAndInc 300
      }

      event waitAndInc: int64

      on waitAndInc fn (ms: int64) {
        wait(ms)
        let i = namespace('foo').getOr('inc', 0)
        i = i + 1
        print(i)
        namespace('foo').set('inc', i)
        if i == 3 {
          emit exit 0
        }
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  DSOUTPUT="false
true
baz
false

1
2
3"

  It "runs js"
    When run node temp.js
    The output should eq "$DSOUTPUT"
  End

  It "runs agc"
    Pending rust-implementation
    When run alan-runtime run temp.agc
    The output should eq "$DSOUTPUT"
  End
End
