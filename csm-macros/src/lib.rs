extern crate proc_macro;
use std::{collections::HashMap, fs, io::Write, path::Path};

use lightningcss::{
    bundler::{Bundler, FileProvider},
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions},
};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::Parse, parse_macro_input};

const OUTPUT_DIR: &str = "./target/csm";

fn write(path: &Path, def: &str) {
    let mut f = fs::File::create(path).expect("failed to create file");
    f.write(def.as_bytes()).expect("failed to write file");
}

fn write_bundle(path: &Path) {
    let abs_out = std::fs::canonicalize(path).unwrap();

    let mut bundle =
        fs::File::create(abs_out.join("bundle.tmp.css")).expect("failed to create file");
    let files = fs::read_dir(abs_out.join("css")).expect("failed to read dir");
    for file in files.filter_map(|file| file.ok()) {
        let modified = file
            .metadata()
            .expect("failed to read metadata")
            .modified()
            .expect("failed to read access time");
        let is_old = modified
            .elapsed()
            .expect("failed to read elapsed time")
            .gt(&std::time::Duration::from_secs(60 * 60 * 24));
        if is_old {
            eprintln!("WARN: {} not being used in a while, consider manually deleting it or it will end up in your bundle. If this is a mistake please open an issue on GitHub.", file.path().to_str().unwrap());
        }

        bundle
            .write_fmt(format_args!(
                "@import \"{}\";\n",
                file.path().to_str().unwrap()
            ))
            .expect("failed to write file");
    }

    let fp = FileProvider::new();
    let mut res = Bundler::new(&fp, None, ParserOptions::default())
        .bundle(&abs_out.join("bundle.tmp.css"))
        .expect("failed to bundle");
    res.minify(MinifyOptions::default())
        .expect("failed to minify");
    let final_bundle = res
        .to_css(PrinterOptions {
            #[cfg(feature = "minify")]
            minify: true,
            ..Default::default()
        })
        .expect("failed to print")
        .code;

    write(&abs_out.join("bundle.css"), &final_bundle);

    //fs::remove_file(path.join("bundle.tmp.css")).expect("failed to remove file");
}

#[proc_macro]
pub fn csm(tokens: TokenStream) -> TokenStream {
    let csm = parse_macro_input!(tokens as Csm);

    // write file
    let out_dir = Path::new(OUTPUT_DIR);
    fs::create_dir_all(out_dir.join("css")).expect("failed to create dir");
    let out_file = out_dir.join("css").join(format!("{}.css", csm.id));
    let css = csm
        .rules
        .0
        .iter()
        .map(|(_, rule)| rule.to_css_class())
        .collect::<Vec<_>>()
        .join("\n");
    write(Path::new(&out_file), &css);

    write_bundle(out_dir);

    // output list of classes
    let classes = csm
        .rules
        .0
        .iter()
        .map(|(_, rule)| rule.class_name())
        .collect::<Vec<_>>()
        .join(" ");

    quote! { #classes }.into()
}

struct Csm {
    id: syn::Ident,
    rules: Rules,
}

impl Parse for Csm {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![,]>()?;
        let rules = input.parse::<Rules>()?;
        Ok(Csm { id, rules })
    }
}

#[derive(Clone, Debug)]
struct Rules(HashMap<String, Rule>);

impl Parse for Rules {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut rules = HashMap::new();
        while !input.is_empty() {
            let rule = input.parse::<Rule>()?;
            let name = rule.prop.clone();
            match rules.insert(name, rule) {
                Some(dup_rule) => {
                    return Err(
                        input.error(format!("duplicate rule for property `{}`", dup_rule.prop,))
                    );
                }
                None => {}
            }
        }
        Ok(Rules(rules))
    }
}

// impl Rules {
//     fn merge(&self, other: &Rules) -> Rules {
//         let mut rules = self.0.clone();
//         for (_, rule) in &other.0 {
//             rules.insert(rule.prop.clone(), rule.clone());
//         }
//         Rules(rules)
//     }
// }

#[derive(Clone, Debug)]
struct Rule {
    prop: String,
    values: Vec<Value>,
}

impl Rule {
    fn to_css(&self) -> String {
        let mut css = String::new();
        css.push_str(self.prop.as_str());
        css.push_str(": ");
        css.push_str(
            self.values
                .iter()
                .map(|v| v.value.clone())
                .collect::<Vec<_>>()
                .join(" ")
                .as_str(),
        );
        css.push_str(";");
        css
    }

    fn to_css_class(&self) -> String {
        let mut css = String::new();

        css.push_str(".");
        css.push_str(&self.class_name());
        css.push_str(" { ");
        css.push_str(&self.to_css());
        css.push_str(" }");
        css
    }

    fn class_name(&self) -> String {
        let mut class_name = String::new();
        class_name.push_str(&self.prop_ident());
        class_name.push_str("_");
        class_name.push_str(
            &self
                .values
                .iter()
                .map(|v| v.key.clone())
                .collect::<Vec<_>>()
                .join("_"),
        );
        class_name
    }

    fn prop_ident(&self) -> String {
        match self.prop.as_str() {
            "display" => "d".to_string(),
            "align-items" => "items".to_string(),
            "justify-content" => "justify".to_string(),
            "height" => "h".to_string(),
            "width" => "w".to_string(),
            "background-color" => "bg".to_string(),
            "border-radius" => "rounded".to_string(),
            "font-size" => "fs".to_string(),
            "padding" => "p".to_string(),
            _ => self.prop.replace("-", "_"),
        }
    }
}

impl Parse for Rule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut prop = String::new();

        while !input.is_empty() && !input.peek(syn::Token![:]) {
            if let Ok(value) = input.parse::<syn::Ident>() {
                prop.push_str(value.to_string().as_str());
            } else if let Ok(_) = input.parse::<syn::Token![-]>() {
                prop.push_str("-");
            } else {
                return Err(input.error(format!(
                    "error parsing property name, expected ident or `-`",
                )));
            }
        }
        input.parse::<syn::Token![:]>()?;

        let mut values = vec![];
        if input.peek(syn::Token![$]) {
            input.parse::<syn::Token![$]>().unwrap();
            let ident = input.parse::<syn::Ident>().map_err(|e| {
                syn::Error::new(
                    Span::call_site(),
                    format!("expected ident after `$`, found `{}`", e),
                )
            })?;
            let val = format!("var(--{})", ident.to_string());
            values.push(Value {
                key: ident.to_string().replace(" ", "_"),
                value: val,
            });
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>().unwrap();
            }
        } else {
            while !input.is_empty() && !input.peek(syn::Token![,]) {
                if let Ok(value) = input.parse::<syn::Ident>() {
                    values.push(Value {
                        key: value.to_string().replace(" ", "_"),
                        value: value.to_string(),
                    });
                } else if let Ok(value) = input.parse::<syn::LitInt>() {
                    let val = NumericValue {
                        value: value.base10_parse::<u32>().expect("not a valid integer"),
                        unit: value.suffix().to_string(),
                    };
                    let val_str = format!("{}{}", val.value, val.unit);
                    values.push(Value {
                        key: val_str.replace(" ", "_"),
                        value: val_str,
                    });
                } else {
                    return Err(input.error(format!("error parsing value for prop: {}", prop)));
                }
            }
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Rule { prop, values })
    }
}

#[derive(Clone, Debug)]
struct Value {
    key: String,
    value: String,
}

struct NumericValue {
    value: u32,
    unit: String,
}

#[proc_macro]
pub fn csm_defs(tokens: TokenStream) -> TokenStream {
    csm_colors_impl(tokens)
}

fn csm_colors_impl(input: TokenStream) -> TokenStream {
    let defs = parse_macro_input!(input as TokenDefs);

    // write file
    let out_dir = Path::new(OUTPUT_DIR);
    fs::create_dir_all(out_dir.join("css")).expect("failed to create dir");
    let out_file = out_dir.join("css").join("_csm_defs.css");
    write(Path::new(&out_file), &defs.to_css());

    write_bundle(out_dir);

    quote! {}.into()
}

#[derive(Debug)]
struct TokenDefs {
    tokens: HashMap<String, TokenDef>,
}

impl Parse for TokenDefs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tokens = HashMap::new();
        while !input.is_empty() {
            let def = input.parse::<TokenDef>()?;
            tokens.insert(def.name.clone(), def);
            if input.is_empty() {
                break;
            }
        }

        Ok(TokenDefs { tokens })
    }
}

impl TokenDefs {
    fn to_css(&self) -> String {
        let mut css = String::new();

        css.push_str(":root {");

        for (_, def) in &self.tokens {
            css.push_str("--");
            css.push_str(def.name.as_str());
            css.push_str(": ");
            css.push_str(def.value.as_str());
            css.push_str(";");
        }

        css.push_str("}");

        css
    }
}

#[derive(Debug)]
struct TokenDef {
    name: String,
    value: String,
}

impl Parse for TokenDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = String::new();
        while !input.is_empty() && !input.peek(syn::Token![:]) {
            if let Ok(value) = input.parse::<syn::Ident>() {
                name.push_str(value.to_string().as_str());
            } else if let Ok(_) = input.parse::<syn::Token![-]>() {
                name.push_str("-");
            } else {
                panic!("unexpected token");
            }
        }
        input.parse::<syn::Token![:]>()?;

        let mut value = String::new();

        if let Ok(_) = input.parse::<syn::Token![$]>() {
            // var reference
            let ident = input.parse::<syn::Ident>()?;
            value.push_str(format!("var(--{})", ident.to_string()).as_str());
        } else {
            while !input.is_empty() && !input.peek(syn::Token![,]) {
                input.parse::<syn::Token![#]>()?;
                value.push_str("#");

                let ident = input.parse::<syn::Ident>()?;
                value.push_str(ident.to_string().as_str());
            }
        }

        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }

        Ok(TokenDef { name, value })
    }
}

// TODO: recipe!{} macro should be rethinked entirely.

// #[proc_macro]
// pub fn recipe(tokens: TokenStream) -> TokenStream {
//     recipe_impl(tokens)
// }
//
// fn recipe_impl(input: TokenStream) -> TokenStream {
//     let recipe = parse_macro_input!(input as Recipe);
//
//     let mut matrix = vec![vec![]];
//     for (_, variant) in &recipe.variants {
//         let mut new_rows = Vec::new();
//         for row in &matrix {
//             for (variant_choice, variant_rules) in variant {
//                 let mut new_row = row.clone();
//                 new_row.push((variant_choice.clone(), variant_rules));
//                 new_rows.push(new_row);
//             }
//         }
//         matrix = new_rows;
//     }
//
//     let macro_name = &recipe.name;
//
//     let mut m_rules = vec![];
//     for row in matrix {
//         let names = row.iter().map(|(name, _)| name).collect::<Vec<_>>();
//         let mut r = recipe.base.clone();
//         for (_, rules) in &row {
//             r = r.merge(rules);
//         }
//         let class_name = format!(
//             "{}__{}",
//             &recipe.name,
//             names
//                 .iter()
//                 .map(|s| s.to_string())
//                 .collect::<Vec<String>>()
//                 .join("_")
//         );
//         let css = format!(
//             ".{} {{\n{}\n}}",
//             class_name,
//             r.0.iter()
//                 .map(|(_, rule)| rule.to_css())
//                 .collect::<Vec<String>>()
//                 .join("\n")
//         );
//         m_rules.push(quote! {
//             (#(#names),*) => {
//                 (#class_name, #css)
//             };
//         });
//     }
//
//     quote! {
//         macro_rules! #macro_name {
//             #(#m_rules)*
//         }
//     }
//     .into()
// }
//
// #[derive(Debug)]
// struct Recipe {
//     name: syn::Ident,
//     base: Rules,
//     variants: Vec<(String, HashMap<String, Rules>)>,
//     defaults: Vec<(String, String)>,
// }
//
// impl Parse for Recipe {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let name = input.parse::<syn::Ident>()?;
//         let mut base = None;
//         let mut variants = vec![];
//         let mut defaults = vec![];
//
//         while !input.is_empty() {
//             if input.peek(syn::Ident) {
//                 let ident = input.parse::<syn::Ident>()?;
//                 input.parse::<syn::Token![:]>()?;
//                 let body;
//                 braced!(body in input);
//
//                 if ident.to_string() == "base" {
//                     base = Some(body.parse::<Rules>()?);
//                 } else if ident.to_string() == "default" {
//                     let ident = body.parse::<syn::Ident>()?;
//                     defaults.push(("default".to_string(), ident.to_string()));
//                 } else if ident.to_string() == "variants" {
//                     while !body.is_empty() {
//                         let variant_name = body.parse::<syn::Ident>()?;
//                         body.parse::<syn::Token![:]>()?;
//                         let variant_body;
//                         braced!(variant_body in body);
//
//                         let mut variant_choices = HashMap::new();
//
//                         while !variant_body.is_empty() {
//                             let ident = variant_body.parse::<syn::Ident>()?;
//                             variant_body.parse::<syn::Token![:]>()?;
//                             let choice_body;
//                             braced!(choice_body in variant_body);
//                             variant_choices
//                                 .insert(ident.to_string(), choice_body.parse::<Rules>()?);
//                             if variant_body.peek(syn::Token![,]) {
//                                 variant_body.parse::<syn::Token![,]>()?;
//                             }
//                         }
//
//                         variants.push((variant_name.to_string(), variant_choices));
//                         if body.peek(syn::Token![,]) {
//                             body.parse::<syn::Token![,]>()?;
//                         }
//                     }
//                 } else {
//                     return Err(syn::Error::new(
//                         Span::call_site(),
//                         format!(
//                             "expected base, variants, or default, got: `{}`",
//                             ident.to_string()
//                         ),
//                     ));
//                 }
//             } else if input.peek(syn::Token![,]) {
//                 input.parse::<syn::Token![,]>()?;
//             } else {
//                 return Err(input.error("unexpected token"));
//             }
//         }
//
//         Ok(Recipe {
//             name,
//             base: base.expect("base is required"),
//             variants,
//             defaults,
//         })
//     }
// }
