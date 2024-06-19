extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Stmt, Expr, ExprPath, ExprIndex, ExprField, Block, ExprUnsafe};

#[proc_macro_attribute]
pub fn my_proc_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function statements
    let stmts = &input.block.stmts;

    // Prepare a vector to collect the instrumented statements
    //let mut instrumented_stmts: Vec<Stmt> = Vec::new();

    // Variable to track if we're inside the start-end block
    let mut in_instrumented_region = false;
    let mut flag: bool = true;

    fn instrument_stmts(stmts: &[Stmt], in_instrumented_region: &mut bool, flag: &mut bool) -> Vec<Stmt> {
        let mut instrumented_stmts = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::Semi(Expr::Call(call), _) => {
                    if let Expr::Path(ExprPath { path, .. }) = &*call.func {
                        if let Some(ident) = path.get_ident() {
                            if ident == "start_atomic" {
                                *in_instrumented_region = true;
                            } else if ident == "end_atomic" {
                                *in_instrumented_region = false;
                            }
                        }
                    }
                }
                Stmt::Semi(Expr::Assign(assign), _) | Stmt::Expr(Expr::Assign(assign)) => {
                    if *in_instrumented_region {
                        match &*assign.left {
                            Expr::Index(ExprIndex { expr,index, .. }) => {
                                // Handle array indexing
                                if let Expr::Path(ExprPath { path, .. }) = &**expr {
                                    if let Some(ident) = path.get_ident() {
                                        let index_expr = quote! { #index };
                                        //let pseudo_lhs = syn::parse_quote! { #path[#index_expr] };
                                        let address_expr = quote!(&#ident[#index_expr] as *const _);
                                        let size_expr = quote!(core::mem::size_of_val(&#ident[#index_expr]));
                                        let print_stmt: Stmt = syn::parse_quote! {
                                            //hprintln!("Variable '{}' at address {:p}, size {}", stringify!(#ident), #address_expr, #size_expr);
                                            save_variables( #address_expr, #size_expr);
                                        };
                                        instrumented_stmts.push(print_stmt);
                                    }
                                }

                            }
                            // Handle simple assignment
                            Expr::Path(ExprPath { path, .. }) => {
                                if let Some(ident) = path.get_ident() {
                                    let address_expr = quote! { & #ident as *const _ };
                                    let size_expr = quote! { core::mem::size_of_val(&#ident) };

                                    // Generate instrumentation statement
                                    let print_stmt: Stmt = syn::parse_quote! {
                                        //hprintln!("Variable '{}' at address {:p}, size {}", stringify!(#ident), #address_expr, #size_expr);
                                        save_variables( #address_expr, #size_expr);
                                    };
                                    instrumented_stmts.push(print_stmt);
                                }
                            }
                            Expr::Unary(unary_expr) if matches!(unary_expr.op, syn::UnOp::Deref(_)) =>{
                                // hprintln!("here for *y = 2");
                                    if let Expr::Path(ExprPath { path, .. }) = &*unary_expr.expr {
                                        if let Some(ident) = path.get_ident() {
                                            let address_expr = quote! {  #ident as *const _ };
                                            let size_expr = quote! { core::mem::size_of_val(#ident) };
        
                                            let print_stmt: Stmt = syn::parse_quote! {
                                                //println!("Dereference Assignment: Address: {}; Size: {}", #address_expr, #size_expr);
                                                //hprintln!("Dereference Assignment: {:?}, size {}", #address_expr, #size_expr);
                                                save_variables( #address_expr, #size_expr);
                                            };
                                            instrumented_stmts.push(print_stmt);
                                        }
                                    }
                            }
                            Expr::Field(ExprField{base, member,..})=>{
                                if let Expr::Path(ExprPath { path, .. }) = &**base {
                                    if let Some(base_ident) = path.get_ident() {
                                        //let base_ident_str = base_ident.to_string();
                                        //let member_str = member.to_token_stream().to_string();
                                        let address_expr = quote! { & #base_ident.#member as *const _ };
                                        let size_expr = quote! { core::mem::size_of_val(& #base_ident.#member) };
                                
                                        let print_stmt = syn::parse_quote! {
                                                //hprintln!("Struct Field Assignment: {}.{} at {:p} {}" , #base_ident_str, #member_str ,  #address_expr, #size_expr);
                                                save_variables( #address_expr, #size_expr);
                                            };
                                        instrumented_stmts.push(print_stmt);
                                    }
                                }
                            }
                            
                            _ => {
                                let print_stmt = syn::parse_quote! {
                                    //hprintln!("Struct Field Assignment: {}.{} at {:p} {}" , #base_ident_str, #member_str ,  #address_expr, #size_expr);
                                    hprintln!(" these stmt didn't match {}", &*assign.left.to_token_stream().to_string() );
                                    
                                };
                                instrumented_stmts.push(print_stmt);
                            }
                        }
                    }
                }
                Stmt::Semi(Expr::Unsafe(expr_unsafe),_)=>{
                    let block = &expr_unsafe.block;
                    let instrumented_block = instrument_stmts(&block.stmts, in_instrumented_region, flag);

                    let new_block: Block = syn::parse_quote!({
                        #(#instrumented_block)*
                    });
                    // println!("New stt {}", new_block.to_token_stream().to_string());
                    // instrumented_stmts.push(new_block);
                    
                    instrumented_stmts.push(Stmt::Expr(Expr::Unsafe(ExprUnsafe {
                        attrs: expr_unsafe.attrs.clone(),
                        unsafe_token: expr_unsafe.unsafe_token,
                        block: new_block,
                    })));
                    *flag = false;
                }
                _ => {}
            }
            if *flag{
                instrumented_stmts.push(stmt.clone());
            }else{
                *flag =true;
            }
        }
        instrumented_stmts
    }
    //Recreate the function with instrumented statements
    
    let instrumented_stmts = instrument_stmts(stmts, &mut in_instrumented_region, &mut flag);

    let new_block = syn::parse_quote!({
        #(#instrumented_stmts)*
    });

    let new_fn = ItemFn {
        block: Box::new(new_block),
        ..input
    };

    TokenStream::from(quote!(#new_fn))
}

