use std::io::Read;

use vfs::{FileSystem, MemoryFS};
use vfs_shadow::load_into_vfs;

pub fn main() {
    let fs = load_into_vfs!("example/vfs", MemoryFS::new()).unwrap();
    assert_eq!(read_all(fs.open_file("/config.toml").unwrap()), include_bytes!("./vfs/config.toml"));
    assert_eq!(read_all(fs.open_file("/data.json").unwrap()), include_bytes!("./vfs/data.json"));
    assert_eq!(read_all(fs.open_file("/README.md").unwrap()), include_bytes!("./vfs/README.md"));

    let ellie = read_all(fs.open_file("/some dir/ellie.txt").unwrap());
    assert_eq!(ellie, include_bytes!("./vfs/some dir/ellie.txt"));
    println!("{}", String::from_utf8(ellie).unwrap());

    println!(r"The virtual filesystem works! \o/");
}

fn read_all(mut reader: impl Read) -> Vec<u8> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).unwrap();
    buf
}
