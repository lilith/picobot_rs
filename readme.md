## This is a prototype [picobot](https://www.cs.hmc.edu/picobot/) emulator

Manual click testing is wasteful and a bad habit. 

To run this you'll need to [install Rust](https://www.rustup.rs/). 

Once `rustc --version` works, then: 

1. `git clone https://github.com/nathanaeljones/picobot_rs.git`
2. Edit `src/lib.rs` to use your desired script and map room.
3. Run `cargo test` to exhustively test all starting locations. 
