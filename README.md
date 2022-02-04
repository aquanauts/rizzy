## rizzy

All in a tizzy over timestamps? `rizzy` is a UNIX filter that will
convert UTC timestamps into the timestamp of your choosing.

### Example

Before:

```bash
$ cat some.log.file
2021-09-17 17:28:07+0000 INFO Running on slurm - limited math CPU usage by a SLURM_CPUS_ON_NODE of 3
2021-09-17 17:30:12+0000 INFO Initializing Environment. Using Environment.V5_C_PROD
```

After:
```
$ cat some.log.file | rizzy --chi
2021-09-17T12:28:07-05:00 INFO Running on slurm - limited math CPU usage by a SLURM_CPUS_ON_NODE of 3
2021-09-17T12:30:12-05:00 INFO Initializing Environment. Using Environment.V5_C_PROD
```

### Flags

Running `rizzy --help` gives more information, but basically:
* use `--chi` for Chicago and `--nyc` for New York
* for other zones use `--zone ...`
* To output in other formats than RFC3339 use `--format`
* To optionally treat big numbers encountered as nanos-since-epoch, use `--convert-epoch-nanos`


### Building and running

* Grab `rustup` - either from [the website](https://www.rust-lang.org/tools/install) or by `sudo snap install rustup --classic`.
* Install the stable rust version: `rustup install stable`
* Build the code with `cargo build`, or run with `cargo run`


## Making a release

* Bump the version in Cargo.toml
* Run `cargo build` and `cargo test` to make sure everything's working and to ensure the `Cargo.toml` gets updated.
* Commit!
* Push and make sure all's well
* Tag the version with `vXX.YY/ZZ`
* Push the tag (`git push --tags`) which should kick things off automatically!
* Make sure all's well and then mark the release as current in GitHub (with some helpful explanation)
