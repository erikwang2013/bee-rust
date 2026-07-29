// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl bee_config::ConfigSource for #name {
            fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, bee_config::ConfigError> {
                let content = std::fs::read_to_string(&path)
                    .map_err(bee_config::ConfigError::from)?;
                let sections = bee_config::ini::IniParser::parse(&content);
                let default = sections
                    .get("default")
                    .ok_or_else(|| bee_config::ConfigError::MissingKey("default section".into()))?;

                // Build a serde_json::Map, parsing each value as JSON so that
                // numbers, booleans, and null are coerced to their proper types.
                let mut map = serde_json::Map::new();
                for (k, v) in default {
                    let val: serde_json::Value = serde_json::from_str(v)
                        .unwrap_or(serde_json::Value::String(v.clone()));
                    map.insert(k.clone(), val);
                }
                let json_value = serde_json::Value::Object(map);

                serde_json::from_value(json_value)
                    .map_err(|e| bee_config::ConfigError::Deserialize(e.to_string()))
            }

            fn reload(&mut self) -> Result<(), bee_config::ConfigError> {
                Ok(())
            }

            fn watch(&self) -> Result<(), bee_config::ConfigError> {
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
