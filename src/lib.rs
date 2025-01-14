//! # vfs-shadow
//! 
//! This crate allows embedding files from a directory into a virtual filesystem (VFS) during 
//! compile time. 
//! The macro [`load_into_vfs!`] takes a path to a directory and a [filesystem](vfs::FileSystem), 
//! then loads the contents of the directory into the provided filesystem.
//! 
//! ## Usage
//! Use the [`load_into_vfs!`] macro to load files from a directory into a 
//! [MemoryFS](vfs::impls::memory::MemoryFS) (or any other filesystem that implements 
//! [`vfs::FileSystem`]):
//! ```
//! use vfs_shadow::load_into_vfs;
//! use vfs::{MemoryFS, FileSystem};
//! 
//! fn main() {
//!     // Load files into a MemoryFS
//!     let fs = load_into_vfs!("example/vfs", MemoryFS::new()).unwrap();
//! 
//!     // Interact with the embedded files
//!     assert!(fs.exists("/data.json").is_ok());
//! }
//! ```
//! 
//! You can also pass a reference to an existing filesystem:
//! ```
//! use vfs_shadow::load_into_vfs;
//! use vfs::{MemoryFS, FileSystem};
//! 
//! fn main() {
//!     let fs = MemoryFS::new();
//!     load_into_vfs!("example/vfs", &fs).unwrap();
//! 
//!     // Use the filesystem
//!     assert!(fs.exists("/data.json").is_ok());
//! }
//! ```
//! 
//! In both cases, the directory at `example/vfs` is included in the final binary, and its contents 
//! are copied into the provided filesystem.
//! 
//! ### Path Resolution
//! The path provided to `load_into_vfs!` is relative to the manifest directory. 
//! This can change when [`proc_macro::Span::source_file`] stabilizes in future.
//! 
//! ## Return Value
//! The macro returns a [`vfs::VfsResult<()>`]. 
//! If an error occurs while copying files into the filesystem, the operation will fail, and 
//! execution will stop.

use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use proc_macro_error2::proc_macro_error;
use quote::quote;
use std::path::PathBuf;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

mod files;

/// Embeds files from a directory into a virtual filesystem at compile time.
/// 
/// For general usage of the `load_into_vfs!` macro, please refer to the 
/// [crate-level documentation](crate).
/// 
/// ## Pseudo Signature
/// ```rust,ignore
/// load_into_vfs!<FS: vfs::FileSystem>(path: &str, fs: &FS) -> vfs::VfsResult<FS>
/// ```
/// 
/// ## Implementation Details
/// The `load_into_vfs!` macro is designed to load the contents of a directory into a virtual 
/// filesystem at compile time. 
/// Here’s how it works:
/// 
/// 1. **Directory Walking**: 
///     During compile time, the specified directory is recursively walked. 
///     The macro collects the absolute paths of all entries within the directory, along with 
///     whether each entry is a file or a directory.
/// 
/// 2. **Trait Generation**:
///     The macro generates a private trait called `LoadIntoFileSystem`, which is implemented for 
///     any type that implements the [`vfs::FileSystem`] trait. 
///     This trait includes a single function, `load_into_vfs`, which is generated at compile time.
/// 
/// 3. **Directory Handling**:
///     For directories, the generated code will create the corresponding directory in the VFS 
///     using:
///     ```rust,ignore
///     self.create_dir(#vfs_path)?;
///     ```
/// 
/// 4. **File Handling**:
///     For files, the macro generates a block of code to include the file contents as bytes and 
///     write them into the VFS:
///     ```rust,ignore
///     {
///         static BYTES: &[u8] = ::std::include_bytes!(#real_path);
///         let mut file = self.create_file(#vfs_path)?;
///         file.write_all(BYTES)?;
///     }
///     ```
///     This ensures the file's bytes are included in the binary and written to the VFS at runtime.
/// 
/// 5. **Returning the Filesystem**:
///     After processing all entries, the macro returns the filesystem wrapped in a 
///     [`vfs::VfsResult`], allowing further interactions with the virtual filesystem.
#[proc_macro]
#[proc_macro_error]
pub fn load_into_vfs(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as Args);

    let fs = &args.fs;
    let dir_entries = files::dir_entries(&args.path.0, args.path.1);
    let load_into_vfs = load_into_vfs_tokens(&dir_entries);

    quote! {{
        use ::vfs::FileSystem;

        trait LoadIntoFileSystem: FileSystem {
            #load_into_vfs
        }

        impl<FS> LoadIntoFileSystem for FS where FS: FileSystem {}
        let fs = #fs;
        fs.load_into_vfs().map(|_| fs)
    }}
    .into()
}

struct Args {
    path: (PathBuf, Span2),
    fs: syn::Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: syn::LitStr = input.parse()?;
        let path = (PathBuf::from(path.value()), path.span());
        input.parse::<Token![,]>()?;
        let fs: syn::Expr = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("unexpected tokens"));
        }

        Ok(Args { path, fs })
    }
}

fn load_into_vfs_tokens(dir_entries: &[files::DirEntry]) -> TokenStream2 {
    let mut instructions: Vec<TokenStream2> = Vec::with_capacity(dir_entries.len());
    for files::DirEntry {
        real_path,
        vfs_path,
        file_type,
    } in dir_entries
    {
        let real_path = real_path.display().to_string();
        let vfs_path = vfs_path.display().to_string();

        if file_type.is_dir() {
            instructions.push(quote!(self.create_dir(#vfs_path)?;));
        }

        if file_type.is_file() {
            instructions.push(quote! {{
                static BYTES: &[u8] = ::std::include_bytes!(#real_path);
                let mut file = self.create_file(#vfs_path)?;
                file.write_all(BYTES)?;
            }});
        }
    }

    quote! {
        fn load_into_vfs(&self) -> ::vfs::error::VfsResult<()> {
            #(#instructions)*
            Ok(())
        }
    }
}
