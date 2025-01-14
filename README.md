<h1 align="center">vfs-shadow</h1>
<p align="center">
  <b>
    Effortlessly embed a local directory into a
    <a href="https://crates.io/crates/vfs">vfs</a>.
  </b>
</p>

<br>

<div align="center">

  [![crates.io Version](https://img.shields.io/crates/v/vfs-shadow?style=for-the-badge)](https://crates.io/crates/vfs-shadow)
  [![vfs Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fcptpiepmatz%2Fvfs-shadow%2Fmain%2FCargo.toml&query=dependencies%5B'vfs'%5D.version&prefix=v&style=for-the-badge&label=vfs%20Version)](https://crates.io/crates/vfs)
  [![License](https://img.shields.io/github/license/cptpiepmatz/nu-jupyter-kernel?style=for-the-badge)](https://github.com/cptpiepmatz/vfs-shadow/blob/main/LICENSE)

</div>


## About
`vfs-shadow` is a Rust crate that provides a macro, 
[`load_into_vfs!`](https://docs.rs/vfs-shadow/latest/vfs-shadow/macro.load_into_vfs.html), 
which allows embedding a local directory into a virtual file system using the 
[`vfs`](https://crates.io/crates/vfs) crate.

## Concept
`vfs-shadow` is designed to simplify the process of preparing a virtual file 
system in Rust.
Using the `load_into_vfs!` macro, it allows you to embed a local directory 
directly into your binary at compile time. 
During runtime, the files and directories are automatically written into a 
virtual file system provided by the `vfs` crate, making them accessible as if 
they were part of the file system. 
This makes it easy to bundle assets or configuration files with your application 
without needing to manage them separately. 
By simply pointing to a directory in your project, you can include its contents 
into the virtual file system, streamlining the process of setting up a file 
system for your application.

## Example
In the [example/example.rs](example/example.rs) file, you can see how to use 
the macro to embed a directory into the virtual file system:

```rs
let fs = load_into_vfs!("example/vfs", MemoryFS::new()).unwrap();
```

This will include the [example/vfs](example/vfs) directory into the file system 
and use it with the `MemoryFS` implementation of the `vfs` crate.

## Version Scheme
The version follows the format `x.y.z+a.b`:
- `x.y.z` is the crate version.
- `a.b` is the version of the `vfs` crate.

The patch version is ignored here, as upgrading it *should* not cause issues.

## License 
This crate is licensed under the MIT license. 
See the [LICENSE](LICENSE) file for more details.
