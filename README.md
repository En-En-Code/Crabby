# Overview
Crabby is an original UCI chess engine written in the [Rust programming language](https://www.rust-lang.org/).
Crabby gets its name from the Rust mascot, Ferris the crab.
This project is an experiment to learn a new language and explore chess programming.
I would greatly appreciate all feedback on my progress for both!

# Building
This fork of Crabby should not be dependent on nightly features any longer, so it can be built using the latest stable version of [Cargo](https://github.com/rust-lang/cargo) using the following commands:
```sh
git clone https://github.com/Johnson-A/Crabby.git
cd Crabby
cargo build --release
./target/release/crabby
```

To make use of native CPU features and run the performance benches for some functions, you can run the [Makefile](Makefile) or run the commands in the Makefile.
```
cargo rustc --release --lib -- -C target-feature=+popcnt -C target-cpu=native
cargo rustc --release --bin crabby -- -C target-feature=+popcnt -C target-cpu=native
cargo bench --bench testing
```

# Features
* Nega-Max alpha beta pruning
* Iterative deepening
* PVS
* Quiescence Search
* Null move pruning
* Late Move Reduction
* Killer move heuristic
* Static exchange evaluation
* Transposition Table with Zobrist hashing
* Bitboard based representation
* Magic move generation

# Planned
* Aspiration window
* Time manager -> improvements
* PVS or MTD(f) -> improvements
* Piece-square evaluation
* Evaluation -> improvements
* 50 move rule
* Multi-threaded search
* Parameter Optimization
* UCI option parsing and implementation
* Migrate to criterion for benches

# Extended UCI Commands
* perft x - Run perft to a depth x
* test move - Run perft on many positions to validate move generation
* test perf - Search to a given depth in many positions to test performance

# Thanks
I'd like to thank the [chess programming wiki](https://chessprogramming.wikispaces.com),
the [talk chess forums](http://www.talkchess.com/forum/index.php), and the open source
[Stockfish engine](https://github.com/official-stockfish/Stockfish) for being great resources.

# Terms
Crabby is licensed under the **GNU General Public License** (GPLv2) as specified in the [LICENSE](LICENSE).
