use crate::{bundle::MessageBundle, codegen::CodeGenerator, error::Error};

use proc_macro2::TokenStream;
use quote::quote;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[macro_export]
macro_rules! generate {
    ($fluent_resource_root:expr, $generator:expr, $out_file:expr) => {{
        if let Ok(src) = generate($fluent_resource_root, $generator) {
            let out_dir =
                env::var("OUT_DIR").expect("OUT_DIR environment variables is not defined");
            let destination = Path::new(&out_dir)
                .join("generated")
                .join("fluent")
                .join($out_file);
            fs::create_dir_all(destination.parent().unwrap())
                .expect("Error creating output directory");
            fs::write(destination, src).expect("Error writing generated sources");
        }
    }};
}

pub fn generate(
    root_dir: impl AsRef<Path>,
    code_generator: impl CodeGenerator,
) -> Result<String, Error> {
    println!(
        "cargo:rerun-if-changed={}",
        root_dir.as_ref().to_string_lossy()
    );
    let paths = list_fluent_resources(&root_dir)?;

    let mut resources = vec![];
    for res_path in paths {
        let path = res_path.strip_prefix(root_dir.as_ref())?.into();
        let content = fs::read_to_string(res_path)?;
        resources.push((path, content));
    }

    let mut generated_code: Vec<TokenStream> = vec![];
    let mut errors = vec![];

    for message_bundle_result in create_message_bundles(resources)? {
        match message_bundle_result {
            Ok(message_bundle) => match code_generator.generate(&message_bundle) {
                Ok(tokens) => generated_code.push(tokens),
                Err(e) => {
                    println!("cargo:error={}", e.to_string());
                    errors.push(e);
                }
            },
            Err(Error::MessageBundleValidationError {
                bundle,
                path,
                entries,
            }) => {
                let langs = |s: &BTreeSet<String>| {
                    s.iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                };
                for entry in &entries {
                    let defined = langs(&entry.defined_in_languages);
                    let undefined = langs(&entry.undefined_in_languages);
                    println!("cargo:error=Localization bundle '{}' {} consistency validation error: Message '{}' is defined for {} but not for {}", 
                        bundle, path, entry.message.name, defined, undefined);
                }

                errors.push(Error::MessageBundleValidationError {
                    bundle,
                    path,
                    entries,
                });
            }
            Err(e) => {
                println!("cargo:error={}", e.to_string());
                errors.push(e);
            }
        }
    }

    if errors.is_empty() {
        Ok(quote! {
            #(#generated_code)*
        }
        .to_string())
    } else {
        Err(errors.into_iter().next().unwrap())
    }
}

fn create_message_bundles(
    resources: Vec<(PathBuf, String)>,
) -> Result<Vec<Result<MessageBundle, Error>>, Error> {
    let bundles_by_path = resources.into_iter().try_fold(
        BTreeMap::<PathBuf, Vec<(String, String)>>::new(),
        |mut acc,
         (resource_path, resource_content)|
         -> Result<BTreeMap<PathBuf, Vec<(String, String)>>, Error> {
            let mut path_components = resource_path.components();
            let lang = path_components
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .ok_or_else(|| Error::InvalidPath(resource_path.clone()))?
                .to_string();
            let bundle_path = path_components.as_path().to_path_buf();

            acc.entry(bundle_path)
                .or_default()
                .push((lang, resource_content));

            Ok(acc)
        },
    )?;

    Ok(bundles_by_path
        .into_iter()
        .map(|(bundle_path, lang_resources)| {
            let bundle_name = bundle_path
                .file_stem()
                .ok_or_else(|| Error::InvalidPathFormat(bundle_path.clone()))?
                .to_str()
                .ok_or_else(|| Error::InvalidPath(bundle_path.clone()))?;

            MessageBundle::create(&bundle_name, &bundle_path, lang_resources)
        })
        .collect())
}

fn is_fluent_resource(path: &PathBuf) -> bool {
    path.extension()
        .is_some_and(|ext| ext.to_string_lossy() == "ftl")
}

fn list_fluent_resources(root_dir: &impl AsRef<Path>) -> Result<Vec<PathBuf>, Error> {
    let mut pending: Vec<PathBuf> = vec![root_dir.as_ref().into()];
    let mut resource_paths: Vec<PathBuf> = vec![];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir)? {
            let entry_path = entry?.path();
            if entry_path.is_dir() {
                pending.push(entry_path);
            } else if is_fluent_resource(&entry_path) {
                resource_paths.push(entry_path);
            }
        }
    }
    resource_paths.sort();
    Ok(resource_paths)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{
        bundle::{LanguageBundle, MessageBundle},
        message::{Message, Var},
    };

    fn make_fluent_resources() -> Vec<(PathBuf, String)> {
        vec![
            (
                PathBuf::from_str("en/main.ftl").unwrap(),
                "hello=Hello { $name }".to_string(),
            ),
            (
                PathBuf::from_str("ru-RU/main.ftl").unwrap(),
                "hello=Привет, { $name }".to_string(),
            ),
            (
                PathBuf::from_str("en/extra/test.ftl").unwrap(),
                "greetings=Greetings { $user }".to_string(),
            ),
            (
                PathBuf::from_str("ru-RU/extra/test.ftl").unwrap(),
                "greetings=Привет, { $user }".to_string(),
            ),
        ]
    }

    #[test]
    fn create_message_bundles() {
        let resources = make_fluent_resources();

        let bundles = super::create_message_bundles(resources.clone()).unwrap();
        let mut actual = bundles.iter();

        assert_eq!(
            &MessageBundle {
                name: "test".to_string(),
                path: PathBuf::from_str("extra/test.ftl").unwrap(),
                language_bundles: vec![
                    LanguageBundle {
                        language: "en".to_string(),
                        resource: resources[2].1.clone(),
                        messages: vec![Message {
                            name: "greetings".to_string(),
                            vars: vec![Var {
                                name: "user".to_string(),
                            }]
                            .into_iter()
                            .collect(),
                            attrs: None,
                            attribute_name: None,
                        }]
                        .into_iter()
                        .collect(),
                    },
                    LanguageBundle {
                        language: "ru-RU".to_string(),
                        resource: resources[3].1.clone(),
                        messages: vec![Message {
                            name: "greetings".to_string(),
                            vars: vec![Var {
                                name: "user".to_string(),
                            }]
                            .into_iter()
                            .collect(),
                            attrs: None,
                            attribute_name: None,
                        }]
                        .into_iter()
                        .collect(),
                    },
                ],
            },
            actual.next().unwrap().as_ref().unwrap(),
        );

        assert_eq!(
            &MessageBundle {
                name: "main".to_string(),
                path: PathBuf::from_str("main.ftl").unwrap(),
                language_bundles: vec![
                    LanguageBundle {
                        language: "en".to_string(),
                        resource: resources[0].1.clone(),
                        messages: vec![Message {
                            name: "hello".to_string(),
                            vars: vec![Var {
                                name: "name".to_string(),
                            }]
                            .into_iter()
                            .collect(),
                            attrs: None,
                            attribute_name: None,
                        }]
                        .into_iter()
                        .collect(),
                    },
                    LanguageBundle {
                        language: "ru-RU".to_string(),
                        resource: resources[1].1.clone(),
                        messages: vec![Message {
                            name: "hello".to_string(),
                            vars: vec![Var {
                                name: "name".to_string(),
                            }]
                            .into_iter()
                            .collect(),
                            attrs: None,
                            attribute_name: None,
                        }]
                        .into_iter()
                        .collect(),
                    },
                ],
            },
            actual.next().unwrap().as_ref().unwrap(),
        );
    }
}
