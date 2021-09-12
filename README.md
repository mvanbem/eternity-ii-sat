# SAT problem generator and assignment validator for the Eternity II puzzle

Generate the SAT problem (requires 4.7G available in the current directory).

```shell
$ cargo run --release --bin emit_problem > eternity-ii.cnf
```

Solve it with your favorite SAT solver.

```shell
$ kissat eternity-ii.cnf | tee eternity-ii.log
```

Convert the satisfying assignment to a URL on https://e2.bucas.name/, a nice web visualizer.
```
$ cargo run --release --bin translate_to_url < eternity-ii.log
```
