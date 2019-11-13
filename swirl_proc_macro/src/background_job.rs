#![warn(warnings)]
use crate::diagnostic_shim::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

pub fn expand(item: syn::ItemFn) -> Result<TokenStream, Diagnostic> {
    let job = BackgroundJob::try_from(item)?;

    let attrs = job.attrs;
    let vis = job.visibility;
    let fn_token = job.fn_token;
    let name = job.name;
    let env_pat = &job.args.env_pat;
    let env_type = &job.args.env_type;
    let fn_args = &job.args;
    let struct_def = job.args.struct_def();
    let struct_assign = job.args.struct_assign();
    let arg_names = job.args.names();
    let return_type = job.return_type;
    let body = job.body;

    let res = quote! {
        #(#attrs)*
        #vis #fn_token #name (#(#fn_args),*) -> #name :: Job {
            #name :: Job {
                #(#struct_assign),*
            }
        }

        impl swirl::Job for #name :: Job {
            type Environment = #env_type;
            const JOB_TYPE: &'static str = stringify!(#name);

            #fn_token perform(self, #env_pat: &Self::Environment) #return_type {
                let Self { #(#arg_names),* } = self;
                #(#body)*
            }
        }

        mod #name {
            use super::*;

            #[derive(swirl::Serialize, swirl::Deserialize)]
            #[serde(crate = "swirl::serde")]
            pub struct Job {
                #(#struct_def),*
            }

            swirl::register_job!(Job);
        }
    };
    Ok(res)
}

struct BackgroundJob {
    attrs: Vec<syn::Attribute>,
    visibility: syn::Visibility,
    fn_token: syn::Token![fn],
    name: syn::Ident,
    args: JobArgs,
    return_type: syn::ReturnType,
    body: Vec<syn::Stmt>,
}

impl BackgroundJob {
    fn try_from(item: syn::ItemFn) -> Result<Self, Diagnostic> {
        let syn::ItemFn {
            attrs,
            vis,
            constness,
            unsafety,
            asyncness,
            abi,
            ident,
            decl,
            block,
        } = item;

        if let Some(constness) = constness {
            return Err(constness
                .span
                .error("#[swirl::background_job] cannot be used on const functions"));
        }

        if let Some(unsafety) = unsafety {
            return Err(unsafety
                .span
                .error("#[swirl::background_job] cannot be used on unsafe functions"));
        }

        if let Some(asyncness) = asyncness {
            return Err(asyncness
                .span
                .error("#[swirl::background_job] cannot be used on async functions"));
        }

        if let Some(abi) = abi {
            return Err(abi
                .span()
                .error("#[swirl::background_job] cannot be used on functions with an abi"));
        }

        if !decl.generics.params.is_empty() {
            return Err(decl
                .generics
                .span()
                .error("#[swirl::background_job] cannot be used on generic functions"));
        }

        if let Some(where_clause) = decl.generics.where_clause {
            return Err(where_clause
                .where_token
                .span
                .error("#[swirl::background_job] cannot be used on functions with a where clause"));
        }

        let fn_token = decl.fn_token;
        let return_type = decl.output.clone();
        let job_args = JobArgs::try_from(*decl)?;

        Ok(Self {
            attrs,
            visibility: vis,
            fn_token,
            name: ident,
            args: job_args,
            return_type,
            body: block.stmts,
        })
    }
}

struct JobArgs {
    env_pat: syn::Pat,
    env_type: syn::Type,
    args: Punctuated<syn::ArgCaptured, syn::Token![,]>,
}

impl JobArgs {
    fn try_from(decl: syn::FnDecl) -> Result<Self, Diagnostic> {
        let mut first_arg = true;
        let mut env_pat = syn::parse_quote!(_);
        let mut env_type = syn::parse_quote!(());
        let mut args = Punctuated::new();

        for fn_arg in decl.inputs {
            let arg_captured = match fn_arg {
                syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                    return Err(fn_arg.span().error("Background jobs cannot take self"));
                }
                syn::FnArg::Inferred(_) | syn::FnArg::Ignored(_) => {
                    unreachable!("would have failed parsing")
                }
                syn::FnArg::Captured(arg_captured) => arg_captured,
            };

            if let syn::Pat::Ident(syn::PatIdent {
                by_ref: None,
                subpat: None,
                ..
            }) = arg_captured.pat
            {
                // ok
            } else {
                return Err(arg_captured
                    .pat
                    .span()
                    .error("#[swirl::background_job] cannot yet handle patterns"));
            }

            if first_arg {
                first_arg = false;
                if let syn::Type::Reference(type_ref) = arg_captured.ty {
                    if let Some(mutable) = type_ref.mutability {
                        return Err(mutable.span.error("Unexpected `mut`"));
                    }
                    env_pat = arg_captured.pat;
                    env_type = *type_ref.elem;
                    continue;
                }
            }

            args.push(arg_captured);
        }

        Ok(Self {
            env_pat,
            env_type,
            args,
        })
    }

    fn struct_def(&self) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.args.iter().map(|arg| quote::quote!(pub(super) #arg))
    }

    fn struct_assign(&self) -> impl Iterator<Item = syn::FieldValue> + '_ {
        self.names().map(|ident| syn::parse_quote!(#ident: #ident))
    }

    fn names(&self) -> impl Iterator<Item = syn::Ident> + '_ {
        self.args.iter().map(|arg| match &arg.pat {
            syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            _ => unreachable!(),
        })
    }
}

impl<'a> IntoIterator for &'a JobArgs {
    type Item = <&'a Punctuated<syn::ArgCaptured, syn::Token![,]> as IntoIterator>::Item;
    type IntoIter = <&'a Punctuated<syn::ArgCaptured, syn::Token![,]> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.args).into_iter()
    }
}
