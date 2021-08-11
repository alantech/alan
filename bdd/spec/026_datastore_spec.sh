Include build_tools.sh

Describe "@std/datastore"
  Describe "distributed kv"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/datastore import namespace, has, set, del, getOr

        on start {
          const ns = namespace('foo');
          print(ns.has('bar'));
          ns.set('bar', 'baz');
          print(ns.has('bar'));
          print(ns.getOr('bar', ''));
          ns.del('bar');
          print(ns.has('bar'));
          print(ns.getOr('bar', ''));

          ns.set('inc', 0);
          emit waitAndInc 100;
          emit waitAndInc 200;
          emit waitAndInc 300;
        }

        event waitAndInc: int64

        on waitAndInc fn (ms: int64) {
          wait(ms);
          let i = namespace('foo').getOr('inc', 0);
          i = i + 1 || 0;
          print(i);
          namespace('foo').set('inc', i);
          if i == 3 {
            emit exit 0;
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
      When run test_js
      The output should eq "$DSOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$DSOUTPUT"
    End
  End

  Describe "distributed compute"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/datastore import namespace, set, ref, mut, with, run, mutOnly, closure, getOr

        on start {
          // Initial setup
          const ns = namespace('foo');
          ns.set('foo', 'bar');

          // Basic remote execution
          const baz = ns.ref('foo').run(fn (foo: string) = foo.length());
          print(baz);

          // Closure-based remote execution
          let bar = 'bar';
          const bay = ns.ref('foo').closure(fn (foo: string): int64 {
            bar = 'foobar: ' + foo + bar;
            return foo.length();
          });
          print(bay);
          print(bar);

          // Constrained-closure that only gets the 'with' variable
          const bax = ns.ref('foo').with(bar).run(fn (foo: string, bar: string): int64 = #foo +. #bar);
          print(bax);

          // Mutable closure
          const baw = ns.mut('foo').run(fn (foo: string): int64 {
            foo = foo + 'bar';
            return foo.length();
          });
          print(baw);

          // Mutable closure that affects the foo variable
          const bav = ns.mut('foo').closure(fn (foo: string): int64 {
            foo = foo + 'bar';
            bar = bar * foo.length();
            return bar.length();
          });
          print(bav);
          print(bar);

          // Constrained mutable closure that affects the foo variable
          const bau = ns.mut('foo').with(bar).run(fn (foo: string, bar: string): int64 {
            foo = foo * #bar;
            return foo.length();
          });
          print(bau);

          // 'Pure' function that only does mutation
          ns.mut('foo').mutOnly(fn (foo: string) {
            foo = foo + foo;
          });
          print(ns.getOr('foo', 'not found'));

          // Constrained 'pure' function that only does mutation
          ns.mut('foo').with(bar).mutOnly(fn (foo: string, bar: string) {
            foo = foo + bar;
          });
          print(ns.getOr('foo', 'not found'));

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    DCOUTPUT="3
3
foobar: barbar
17
6
126
foobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbar
1134
barbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbar
barbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbar"

    It "runs js"
      When run test_js
      The output should eq "$DCOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$DCOUTPUT"
    End
  End

End
