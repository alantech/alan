# alan

This repository houses all the components for the Alan programming language.

## Install
```
make clean
make build
make install
```

## Usage

```
alan build <sourcefile>
alan run out.agc
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

To run a single test group use the line number the `Describe`:
```
make bdd testfile=bdd/spec/001_event_spec.sh:30
```