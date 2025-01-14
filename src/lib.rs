use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error2::proc_macro_error;
use quote::quote;
use std::path::PathBuf;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

mod files;

#[proc_macro]
#[proc_macro_error]
pub fn load_into_vfs(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as Args);

    let fs = &args.fs;
    let dir_entries = files::dir_entries(&args.path).unwrap();
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
    path: PathBuf,
    fs: syn::Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: syn::LitStr = input.parse()?;
        let path = PathBuf::from(path.value());
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
    } in dir_entries.iter()
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
