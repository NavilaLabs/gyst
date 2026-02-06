use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Path, parse::Parse, parse::ParseStream, parse_macro_input};

struct ModuleArgs {
    module_path: Path,
}

impl Parse for ModuleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let module_path = input.parse()?;
        Ok(ModuleArgs { module_path })
    }
}

/// A procedural macro to wrap a test function with setup/teardown logic provided by a module.
///
/// Usage: #[with_lifecycle(my_test_setup_module)]
///
/// The module must provide two public functions:
/// - `pub fn before()`
/// - `pub fn after()`
#[proc_macro_attribute]
pub fn with_lifecycle(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ModuleArgs);
    let module = args.module_path;

    let input_fn = parse_macro_input!(input as ItemFn);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let body = &input_fn.block;
    let attrs = &input_fn.attrs;

    // We use an RAII (Resource Acquisition Is Initialization) pattern here.
    // We create a local struct that implements Drop. Its drop method calls module::after().
    // This ensures module::after() runs even if the test panics or returns early.
    // module::before() is called immediately before creating the guard.
    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            struct __ScopeGuard;

            impl Drop for __ScopeGuard {
                fn drop(&mut self) {
                    #module::after();
                }
            }

            // Run the setup
            #module::before();

            // Initialize the guard. When this variable goes out of scope (end of function or panic),
            // module::after() will be called.
            let __guard = __ScopeGuard;

            // Execute the original function body.
            // Because #body is a block { ... }, it acts as an expression.
            // Its return value will be returned by the outer function.
            #body
        }
    };

    TokenStream::from(expanded)
}
