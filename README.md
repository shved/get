A terrible code made just to get comfortible with rust semantics and std lib facilities.

## TODO:
- [X] init
- [X] refactor it to split bin command and lib implementation
- [X] commit
- [X] refactor error handling so every call to lib returns result to the top level and panics are handled with proper exit code in main.rs and all the success messages are printed by main.rs. fight unwraps
- [X] start doing tests ^_^
- [X] restore
- [X] extract all the path variables to call config to not pass root_path through all the calls
- [X] make it compare ignore entries to path segments to equality (https://doc.rust-lang.org/std/path/struct.Path.html#method.components)
- [X] make it work from any place, not just repo root directory (using Path.ancestors, check it in paths mod)
- [X] .get.toml file with all the repo preferences (including getignore) and store options in static segment
- [X] rework it all to work with structs, remove shared state
- [ ] change default commit message to something sensible (a timestamp? files changed?)
- [ ] rework tests to use `ramfs`
- [ ] add doc comments
- [ ] remake blob content to be a byte slice to support arbitrary binary data, not just utf-8 text files.
- [ ] commit log
- [ ] delete last commit
- [ ] diff
- [ ] branches
- [ ] research and maybe set update timestamp to restored files to the time from extra gzip header segment
- [ ] push/pull via ssh + conflicts detection
- [ ] handle interrupt signal trying to clean up after the current job is interrupted
- [ ] command to delete dangling objects (gc)
- [ ] support multiline commit message
- [ ] deal with empty folders (not needed)
- [ ] lock repo with .get/lock file
