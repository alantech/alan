# alan ![CI](https://github.com/alantech/alan/workflows/CI/badge.svg)

This repository houses all the components for the Alan programming language.

## Install
```
make clean
make install # currently doesn't work quite right in Linux if you're using an nvm-managed Node.js (the compiler fails to install if you prefix this last one with sudo)
```

## Usage

To compile to Alan GraphCode:
```
./alan compile <sourcefile>.ln
./alan run out.agc
```

To transpile to Alan's intermediate representation, `alan--`:
```
./alan transpile-amm <sourcefile>.ln
```

To transpile to Javascript:
```
./alan transpile-js <sourcefile>.js
node out.js
```

## Integration tests

Integration tests are in `/bdd` and defined using [Shellspec](https://shellspec.info/). To run all integration tests:
```
make bdd
```

To run a single test file:
```
make bdd testfile=bdd/spec/001_event_spec.sh
```

To run a single test group use the line number corresponding to a `Describe`:
```
make bdd testfile=bdd/spec/001_event_spec.sh:30
```