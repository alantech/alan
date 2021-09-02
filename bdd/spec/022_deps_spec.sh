Include build_tools.sh

Describe "@std/deps"
  Describe "package dependency add"
    before() {
      sourceToAll "
        from @std/deps import Package, install, add, commit, dependency, using, block, fullBlock

        on install fn (package: Package) = package
          .using(['@std/app', '@std/cmd'])
          .dependency('https://github.com/alantech/hellodep.git#deps-perm')
            .add()
          .block('@std/tcp')
          .fullBlock('@std/httpcommon')
          .commit()
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    after_each() {
      rm -r ./dependencies
    }
    After after_each

    has_dependencies() {
      test -d "./dependencies"
    }

    has_alantech() {
      test -d "./dependencies/alantech"
    }

    has_hellodep() {
      test -d "./dependencies/alantech/hellodep"
    }

    has_index() {
      test -f "./dependencies/alantech/hellodep/index.ln"
    }

    has_nested_dependencies() {
      test -d "./dependencies/alantech/hellodep/dependencies"
    }

    has_nested_alantech() {
      test -d "./dependencies/alantech/hellodep/dependencies/alantech"
    }

    has_nested_hellodep() {
      test -d "./dependencies/alantech/hellodep/dependencies/alantech/nestedhellodep"
    }

    has_nested_index() {
      test -f "./dependencies/alantech/hellodep/dependencies/alantech/nestedhellodep/index.ln"
    }

    has_modules() {
      test -d "./dependencies/modules"
    }

    has_std() {
      test -d "./dependencies/modules/std"
    }

    not_has_cmd() {
      if [ -d ./dependencies/modules/std/cmd ]; then
        return 1
      fi
      return 0
    }

    has_pkg_block() {
      test -d "./dependencies/modules/std/tcp"
    }

    has_pkg_full_block_applied() {
      test -d "./dependencies/alantech/hellodep/modules/std/httpcommon" && grep -R -q "export const mock = true" "./dependencies/alantech/hellodep/modules/std/httpcommon/index.ln"
    }

    run_js() {
      node test_$$/temp.js | head -1
    }

    run_agc() {
      alan run test_$$/temp.agc | head -1
    }

    It "runs js"
      When run run_js
      The output should eq "Cloning into './dependencies/alantech/hellodep'..."
      The error should eq "Cloning into './dependencies/alantech/hellodep'..."
      Assert has_dependencies
      Assert has_alantech
      Assert has_hellodep
      Assert has_index
      Assert has_nested_dependencies
      Assert has_nested_alantech
      Assert has_nested_hellodep
      Assert has_nested_index
      Assert has_modules
      Assert has_std
      Assert not_has_cmd
      Assert has_pkg_block
      Assert has_pkg_full_block_applied
    End

    It "runs agc"
      When run run_agc
      The output should eq "Cloning into './dependencies/alantech/hellodep'..."
      Assert has_dependencies
      Assert has_alantech
      Assert has_hellodep
      Assert has_index
      Assert has_nested_dependencies
      Assert has_nested_alantech
      Assert has_nested_hellodep
      Assert has_nested_index
      Assert has_modules
      Assert has_std
      Assert not_has_cmd
      Assert has_pkg_block
      Assert has_pkg_full_block_applied
    End
  End
End