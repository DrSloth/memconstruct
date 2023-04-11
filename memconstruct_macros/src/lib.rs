use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{Data, DataStruct, DeriveInput, Fields, Generics, Ident, Type, Visibility, Member, Index};

#[proc_macro_derive(MemConstruct)]
pub fn memconstruct_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let item_name = input.ident;
    let data = input.data;

    let module_name = Ident::new(
        &format!("__memconstruct__impl__{}", item_name),
        item_name.span(),
    );
    let impl_tokens = match data {
        Data::Struct(data_struct) => memconstruct_derive_struct_impl(
            item_name.clone(),
            input.generics,
            data_struct,
            input.vis,
        ),
        data => todo!("Currently not supported: {:?}", data),
    };

    let expanded = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #module_name {
            #![allow(clippy::all, warnings, unused, non_snake_case, non_camel_case_types)]
            use super:: #item_name ;
            #impl_tokens
        }
    };

    // panic!("{}", expanded);

    expanded.into()
}

fn memconstruct_derive_struct_impl(
    name: Ident,
    generics: Generics,
    data_struct: DataStruct,
    vis: Visibility,
) -> impl ToTokens {
    let constructor_name = Ident::new(&format!("{}MemConstructor", name), name.span());

    let fields = match &data_struct.fields {
        // A struct where all fields are named (filter_map won't skip anythin)
        Fields::Named(fields) => fields
            .named
            .iter()
            .filter_map(|field| {
                Some(MemConstructField {
                    name: Member::Named(field.ident.clone()?),
                    field_type: field.ty.clone(),
                })
            })
            .collect::<Vec<_>>(),
        // A struct where all fields are unnamed
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, field)| MemConstructField {
                name: Member::Unnamed(Index::from(i)),
                field_type: field.ty.clone(),
            })
            .collect::<Vec<_>>(),
        Fields::Unit => return impl_zst(name, constructor_name, generics, quote! { Self }),
    };

    impl_struct(name, constructor_name, generics, &fields, vis)
}

struct MemConstructField {
    name: Member,
    field_type: Type,
}

fn impl_struct(
    name: Ident,
    constructor_name: Ident,
    generics: Generics,
    fields: &[MemConstructField],
    vis: Visibility,
) -> TokenStream2 {
    if fields.is_empty() {
        return impl_zst(name, constructor_name, generics, quote! { Self {} });
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let construction_tokens = fields
        .iter()
        .map(|field| memconstruct_token(&name, &field.name))
        .collect::<Vec<_>>();
    let finished_tokens = fields.iter().map(|_| quote! { (), });
    // The tokens T0 til TN used to mark DC vals on the impls
    let impl_token_generics = (0..fields.len())
        .map(|i| {
            let ident = quote::format_ident!("T{}", i);
            quote! { #ident }
        })
        .collect::<Vec<_>>();
    let mut impls = Vec::with_capacity(fields.len());

    // Create all impl blocks
    for (i, field) in fields.iter().enumerate() {
        let impl_token_generics = impl_token_generics
            .get(0..impl_token_generics.len().saturating_sub(1))
            .unwrap_or_else(|| {
                unreachable!(
                    "This cuts of the last Token from the Vector, this code won't be \
                    reached for empty Vectors"
                )
            });
        // let after = fields
        //     .len()
        //     .checked_sub(i)
        //     .unwrap_or_else(|| unreachable!("The enumeration will never reach len"));

        // TODO make heapconstruction composable
        let field_name = &field.name;
        let param_name = quote::format_ident!("val_{}", field_name);
        let field_type = &field.field_type;
        let before_tokens = impl_token_generics
            .iter()
            .clone()
            .take(i)
            .collect::<Vec<_>>();
        let after_tokens = impl_token_generics.iter().skip(i).collect::<Vec<_>>();
        let impl_token_generics = impl_token_generics;
        let construction_token = construction_tokens
            .get(i)
            .unwrap_or_else(|| unreachable!("There should be a construction token for each field"));
        let setter_name = quote::format_ident!("set_{}", field.name);
        let with_pointer_fn_name = quote::format_ident!("set_{}_with_pointer", field_name);
        let impl_quote = quote! {
            impl < #(#impl_token_generics,)* > #constructor_name
                < #(#before_tokens,)* #construction_token,  #(#after_tokens,)* >
            {
                /// Set the value of the field
                pub fn #setter_name(self, #param_name: #field_type)
                 -> #constructor_name<#(#before_tokens,)* (), #(#after_tokens,)*>
                {
                 // SAFETY: we write to the field via addr_of_mut TODO packed types need unaligned
                 unsafe {
                     ::core::ptr::addr_of_mut!((*self.ptr).#field_name).write(#param_name);
                 }
                 #constructor_name::<#(#before_tokens,)* (), #(#after_tokens,)*> {
                     ptr: self.ptr,
                     boo_scary: ::core::marker::PhantomData::default(),
                  }
                }

                /// Set the value of the field through the pointer
                ///
                /// # SAFETY
                ///
                /// This is marked unsafe as we have to rely on the pointer being actually written
                pub unsafe fn #with_pointer_fn_name(self, init: impl FnOnce(*mut #field_type)) {
                    init(::core::ptr::addr_of_mut!((*self.ptr).#field_name))
                }
            }
        };

        impls.push(impl_quote);
    }

    let constructor_visibility = match vis {
        Visibility::Inherited => quote! { pub(super) },
        vis => quote! { #vis },
    };

    quote! {
        #(
            #[allow(non_camel_case_types)]
            #[allow(clippy::all)]
            pub struct #construction_tokens ;
        )*

        #[allow(non_camel_case_types)]
        #[allow(clippy::all)]
        #constructor_visibility struct #constructor_name <#(#impl_token_generics,)*> {
            ptr: *mut #name,
            boo_scary: ::core::marker::PhantomData::<(#(#impl_token_generics,)*)>,
        }

        unsafe impl #impl_generics ::memconstruct::MemConstruct for #name #ty_generics
            #where_clause
        {
            type Constructor = #constructor_name <#(#construction_tokens,)*> ;
            type ConstructorFinishedToken = #constructor_name <#(#finished_tokens)*> ;
        }

        unsafe impl #impl_generics ::memconstruct::MemConstructConstructor
            for #constructor_name <#(#construction_tokens,)*> #where_clause
        {
            //TODO make this work for generic structs
            type Target = #name;

            unsafe fn new(ptr: *mut #name) -> Self {
                Self {
                    ptr,
                    boo_scary: ::core::marker::PhantomData::default(),
                }
            }
        }

        #(#impls)*
    }
}

fn memconstruct_token(type_name: &Ident, field_name: &Member) -> TokenStream2 {
    let ident = quote::format_ident!("MemConstruct{}{}", type_name, field_name);
    quote! { #ident }
}

fn impl_zst(
    name: Ident,
    constructor_name: Ident,
    generics: Generics,
    zst_constructions: TokenStream2,
) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        unsafe impl ::memconstruct::MemConstruct for #name #ty_generics #where_clause {
            type Constructor = #constructor_name;
            type ConstructorFinishedToken = Self::Constructor;

            fn new_boxed_zst() -> Box<Self> where Self: Sized {
                Box::new( #zst_constructions )
            }
        }

        pub(crate) struct #constructor_name;

        unsafe impl #impl_generics ::memconstruct::MemConstructConstructor for
            #constructor_name #ty_generics #where_clause
        {
            type Target = #name #impl_generics ;

            unsafe fn new(_ptr: *mut Self::Target) -> Self {
                Self
            }
        }
    }
}
