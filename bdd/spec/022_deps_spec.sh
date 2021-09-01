Include build_tools.sh

Describe "@std/deps"
  Describe "package dependency add"
    before() {
      # TODO: Add bdd tests for using, block and fullBlocks
      sourceToAll "
        from @std/deps import Package, install, add, commit, dependency

        on install fn (package: Package) = package
          .dependency('git@github.com:alantech/hellodep.git#deps-perm')
            .add()
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
      echo "--------"
      ls
      ls -la dependencies
      test -d "./dependencies"
    }

    has_alantech() {
      echo "--------"
      ls
      ls -la dependencies
      ls -la dependencies/alantech
      test -d "./dependencies/alantech"
    }

    has_hellodep() {
      echo "--------"
      ls
      ls -la dependencies
      ls -la dependencies/alantech
      ls -la dependencies/alantech/hellodep
      test -d "./dependencies/alantech/hellodep"
    }

    has_index() {
      echo "--------"
      ls
      ls -la dependencies
      ls -la dependencies/alantech
      ls -la dependencies/alantech/hellodep
      test -f "./dependencies/alantech/hellodep/index.ln"
    }

    has_nested_dependencies() {
      ls
      ls -la dependencies
      ls -la dependencies/alantech
      ls -la dependencies/alantech/hellodep
      ls -la dependencies/alantech/hellodep/dependencies
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


    run_js() {
      node test_$$/temp.js | head -1
    }

    run_agc() {
      alan run test_$$/temp.agc | head -1
    }

    It "runs js"
      When run run_js
      sleep 1
      The output should eq "Cloning into './dependencies/alantech/hellodep'..."
      The error should eq "Cloning into './dependencies/alantech/hellodep'..."
      Assert has_dependencies
      Assert has_alantech
      Assert has_hellodep
      Assert has_index
      # Assert has_nested_dependencies
      # Assert has_nested_alantech
      # Assert has_nested_hellodep
      # Assert has_nested_index
    End

    It "runs agc"
      When run run_agc
      sleep 1
      The output should eq "Cloning into './dependencies/alantech/hellodep'..."
      Assert has_dependencies
      Assert has_alantech
      Assert has_hellodep
      Assert has_index
      # Assert has_nested_dependencies
      # Assert has_nested_alantech
      # Assert has_nested_hellodep
      # Assert has_nested_index
    End
  End

  # Describe "package using"
  #   before() {
  #     sourceToAll "
  #       from @std/app import print
  #       from @std/cmd import exec
  #       from @std/deps import Package, install, add, commit, dependency, using

  #       on install fn (package: Package) = package
  #         .using(['@std/app', '@std/cmd'])
  #         .commit()
        
  #       const lsRes = exec('ls');
  #       print(lsRes.stdout);
  #     "
  #   }
  #   BeforeAll before

  #   after() {
  #     cleanTemp
  #   }
  #   AfterAll after

  #   after_each() {
  #     ls
  #     ls -la ./dependencies
  #     rm -r ./dependencies
  #   }
  #   After after_each

  #   has_dependencies() {
  #     test -d "./dependencies"
  #   }

  #   has_modules() {
  #     test -d "./dependencies/modules"
  #   }

  #   has_std() {
  #     test -d "./dependencies/modules/std"
  #   }

  #   has_cmd() {
  #     test -d "./dependencies/modules/std/cmd"
  #   }

  #   has_index() {
  #     test -f "./dependencies/modules/std/cmd/index.ln"
  #   }

  #   run_js() {
  #     node test_$$/temp.js | head -1
  #   }

  #   run_agc() {
  #     alan run test_$$/temp.agc | head -1
  #   }

  #   It "runs js"
  #     When run run_js
  #     Assert has_dependencies
  #     Assert has_modules
  #     Assert has_std
  #     Assert has_cmd
  #     Assert has_index
  #   End

  #   It "runs agc"
  #     When run run_agc
  #     Assert has_dependencies
  #     Assert has_modules
  #     Assert has_std
  #     Assert has_cmd
  #     Assert has_index
  #   End
  # End
End