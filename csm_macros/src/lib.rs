extern crate proc_macro;
use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{braced, parse::Parse, parse_macro_input};

#[proc_macro]
pub fn csm(tokens: TokenStream) -> TokenStream {
    //let tokens: proc_macro2::TokenStream = tokens.into();
    csm_impl(tokens)
}

fn csm_impl(input: TokenStream) -> TokenStream {
    let rules = parse_macro_input!(input as Rules);

    let class_names = rules
        .0
        .iter()
        .map(|rule| rule.class_name())
        .collect::<Vec<String>>()
        .join(" ");

    let css = rules
        .0
        .iter()
        .map(|rule| rule.to_css_class())
        .collect::<Vec<String>>()
        .join("\n");

    quote! {{
        (#class_names, #css)
    }}
    .into()
}

#[derive(Clone, Debug)]
struct Rules(Vec<Rule>);

impl Parse for Rules {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut rules = Vec::new();
        while !input.is_empty() {
            rules.push(input.parse::<Rule>()?);
            if input.is_empty() {
                break;
            }
        }
        Ok(Rules(rules))
    }
}

impl Rules {
    fn merge(&self, other: &Rules) -> Rules {
        let mut rules = self.0.clone();
        for rule in &other.0 {
            rules.push(rule.clone());
        }
        Rules(rules)
    }
}

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

    let css = defs.to_css();

    quote! {{
        #css
    }}
    .into()
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
            input.parse::<syn::Token![,]>()?;
        } else {
            while !input.is_empty() && !input.peek(syn::Token![,]) {
                input.parse::<syn::Token![#]>()?;
                value.push_str("#");

                let ident = input.parse::<syn::Ident>()?;
                value.push_str(ident.to_string().as_str());
            }

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(TokenDef { name, value })
    }
}

#[proc_macro]
pub fn recipe(tokens: TokenStream) -> TokenStream {
    recipe_impl(tokens)
}

fn recipe_impl(input: TokenStream) -> TokenStream {
    let recipe = parse_macro_input!(input as Recipe);

    // let mut rules = recipe.base.0.clone();
    //
    // for (name, variant) in &recipe.variants {
    //     if let Some(variant_rules) = variant.0.clone() {
    //         for rule in variant_rules {
    //             rules.push(rule);
    //         }
    //     }
    // }
    //
    // let mut css = String::new();
    //
    // for rule in rules {
    //     css.push_str(rule.to_css().as_str());
    //     css.push_str("\n");
    // }
    //
    // let mut defaults = HashMap::new();
    //
    // for (name, default) in &recipe.defaults {
    //     defaults.insert(name.clone(), default.clone());
    // }
    //
    // quote! {{
    //     (#css, #defaults)
    // }}
    // .into()

    // variants = [a=1,2,  b=1,2,  c=1]
    // matrix = [
    //            [a=1,    b=1,    c=1],
    //            [a=1,    b=2,    c=1],
    //            [a=2,    b=1,    c=1],
    //            [a=2,    b=2,    c=1],
    //          ]

    let mut matrix = vec![vec![]];
    for (_, variant) in &recipe.variants {
        let mut new_rows = Vec::new();
        for row in &matrix {
            for (variant_choice, variant_rules) in variant {
                let mut new_row = row.clone();
                new_row.push((variant_choice.clone(), variant_rules));
                new_rows.push(new_row);
            }
        }
        matrix = new_rows;
    }

    eprintln!("{:?}", matrix);

    let macro_name = &recipe.name;
    // quote! {
    //     macro_rules! #macro_name {
    //         #(
    //             (#(#matrix),*) => {
    //                 "heh";
    //             }
    //         )*
    //     }
    // }

    let mut m_rules = vec![];
    for row in matrix {
        let names = row.iter().map(|(name, _)| name).collect::<Vec<_>>();
        let mut r = recipe.base.clone();
        for (_, rules) in &row {
            r = r.merge(rules);
        }
        let class_name = format!(
            "{}__{}",
            &recipe.name,
            names
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("_")
        );
        let css = format!(
            ".{} {{\n{}\n}}",
            class_name,
            r.0.iter()
                .map(|rule| rule.to_css())
                .collect::<Vec<String>>()
                .join("\n")
        );
        m_rules.push(quote! {
            (#(#names),*) => {
                (#class_name, #css)
            };
        });
    }

    for x in &m_rules {
        eprintln!("{}", x);
    }

    quote! {
        macro_rules! #macro_name {
            #(#m_rules)*
        }
    }
    .into()
}

#[derive(Debug)]
struct Recipe {
    name: syn::Ident,
    base: Rules,
    variants: Vec<(String, HashMap<String, Rules>)>,
    defaults: Vec<(String, String)>,
}

impl Parse for Recipe {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        let mut base = None;
        let mut variants = vec![];
        let mut defaults = vec![];

        while !input.is_empty() {
            if input.peek(syn::Ident) {
                let ident = input.parse::<syn::Ident>()?;
                input.parse::<syn::Token![:]>()?;
                let body;
                braced!(body in input);

                if ident.to_string() == "base" {
                    base = Some(body.parse::<Rules>()?);
                } else if ident.to_string() == "default" {
                    let ident = body.parse::<syn::Ident>()?;
                    defaults.push(("default".to_string(), ident.to_string()));
                } else if ident.to_string() == "variants" {
                    while !body.is_empty() {
                        let variant_name = body.parse::<syn::Ident>()?;
                        body.parse::<syn::Token![:]>()?;
                        let variant_body;
                        braced!(variant_body in body);

                        let mut variant_choices = HashMap::new();

                        while !variant_body.is_empty() {
                            let ident = variant_body.parse::<syn::Ident>()?;
                            variant_body.parse::<syn::Token![:]>()?;
                            let choice_body;
                            braced!(choice_body in variant_body);
                            variant_choices
                                .insert(ident.to_string(), choice_body.parse::<Rules>()?);
                            if variant_body.peek(syn::Token![,]) {
                                variant_body.parse::<syn::Token![,]>()?;
                            }
                        }

                        variants.push((variant_name.to_string(), variant_choices));
                        if body.peek(syn::Token![,]) {
                            body.parse::<syn::Token![,]>()?;
                        }
                    }
                } else {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!(
                            "expected base, variants, or default, got: `{}`",
                            ident.to_string()
                        ),
                    ));
                }
            } else if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            } else {
                return Err(input.error("unexpected token"));
            }
        }

        Ok(Recipe {
            name,
            base: base.expect("base is required"),
            variants,
            defaults,
        })
    }
}
