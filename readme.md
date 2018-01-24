## This is a prototype [picobot](https://www.cs.hmc.edu/picobot/) emulator

Manual click testing is wasteful and a bad habit. 

To run this you'll need to [install Rust](https://www.rustup.rs/). 

Once `rustc --version` works, then: 

1. `git clone https://github.com/nathanaeljones/picobot_rs.git`
2. Edit `src/lib.rs` to use your desired script and map room.
3. Run `cargo test` to exhustively test all starting locations. 


## License

This program is free software: you can redistribute it and/or modify it 
under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

Nathanael Jones, 2018
Maps from Z Dodds, 2005+
