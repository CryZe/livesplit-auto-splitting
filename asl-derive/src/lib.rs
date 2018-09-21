extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Ident, Lit, LitInt, Meta, Type};

#[proc_macro_derive(ASLState, attributes(Process, Pointer))]
pub fn asl_state(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;
    let process = ast.attrs
        .iter()
        .filter_map(|x| x.interpret_meta())
        .filter_map(|x| match x {
            Meta::NameValue(nv) => Some(nv),
            _ => None,
        })
        .filter(|x| x.ident == "Process")
        .next()
        .unwrap()
        .lit;

    let struct_data = match ast.data {
        Data::Struct(s) => s,
        _ => panic!("Only structs are supported"),
    };

    let (mut pointers, mut fields_current, mut fields_old) = (Vec::new(), Vec::new(), Vec::new());

    for (field_index, field) in struct_data.fields.iter().enumerate() {
        let ident = field.ident.as_ref().unwrap();
        let pointer_path = field
            .attrs
            .iter()
            .filter_map(|x| x.interpret_meta())
            .filter_map(|x| match x {
                Meta::NameValue(nv) => Some(nv),
                _ => None,
            })
            .filter(|x| x.ident == "Pointer")
            .filter_map(|x| match x.lit {
                Lit::Str(s) => Some(s.value()),
                _ => None,
            })
            .next()
            .unwrap();
        let mut splits = pointer_path.split(',').map(|s| s.trim());
        let module_name = splits.next().unwrap();

        let offsets = splits
            .map(|o| syn::parse_str::<LitInt>(o))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let ty = match &field.ty {
            Type::Path(p) => {
                assert_eq!(p.path.segments.len(), 1, "Only builtin types are supported");
                &p.path.segments.last().unwrap().value().ident
            }
            _ => panic!("Unsupported type"),
        };
        let span = ty.span();
        let (ty, call) = match ty.to_string().as_str() {
            "u8" => ("U8", "get_u8"),
            "u16" => ("U16", "get_u16"),
            "u32" => ("U32", "get_u32"),
            "u64" => ("U64", "get_u64"),
            "i8" => ("I8", "get_i8"),
            "i16" => ("I16", "get_i16"),
            "i32" => ("I32", "get_i32"),
            "i64" => ("I64", "get_i64"),
            "f32" => ("F32", "get_f32"),
            "f64" => ("F64", "get_f64"),
            p => panic!("Unsupported type {}", p),
        };
        let ty = Ident::new(ty, span);
        let call = Ident::new(call, span);

        pointers.push(quote! {
            asl::push_pointer_path(#module_name, &[#(#offsets),*], asl::PointerKind::#ty);
        });

        fields_current.push(quote! {
            #ident: asl::#call(#field_index, asl::State::Current)
        });

        fields_old.push(quote! {
            #ident: asl::#call(#field_index, asl::State::Old)
        });
    }

    let tokens = quote! {
        #[no_mangle]
        pub extern "C" fn configure() {
            asl::set_process_name(#process);
            #(#pointers)*
        }

        impl asl::ASLState for #name {
            fn get() -> (Self, Self) {
                (
                    Self {
                        #(#fields_current),*
                    },
                    Self {
                        #(#fields_old),*
                    },
                )
            }
        }
    };

    tokens.into()
}
