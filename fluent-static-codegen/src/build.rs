use crate::{bundle::MessageBundle, codegen::CodeGenerator, error::Error};

use proc_macro2::TokenStream;
use quote::quote;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub fn generate(
    root_dir: impl AsRef<Path>,
    code_generator: impl CodeGenerator,
) -> Result<String, Error> {
    let paths = list_fluent_resources(&root_dir)?;

    let mut resources = vec![];
    for res_path in paths {
        println!("cargo:rerun-if-changed={}", res_path.to_string_lossy());
        let path = res_path.strip_prefix(root_dir.as_ref())?.into();
        let content = fs::read_to_string(res_path)?;
        resources.push((path, content));
    }

    let message_bundle_generated_code = create_message_bundles(resources)?
        .iter()
        .map(|bundle| code_generator.generate(bundle))
        .collect::<Result<Vec<TokenStream>, Error>>()?;

    Ok(quote! {
        #(#message_bundle_generated_code)*
    }
    .to_string())
}

fn create_message_bundles(resources: Vec<(PathBuf, String)>) -> Result<Vec<MessageBundle>, Error> {
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

    bundles_by_path
        .into_iter()
        .map(|(bundle_path, lang_resources)| {
            let bundle_name = bundle_path
                .file_stem()
                .ok_or_else(|| Error::InvalidPathFormat(bundle_path.clone()))?
                .to_str()
                .ok_or_else(|| Error::InvalidPath(bundle_path.clone()))?;

            MessageBundle::create(&bundle_name, &bundle_path, lang_resources)
        })
        .collect()
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

        let expected = vec![
            MessageBundle {
                name: "test".to_string(),
                path: PathBuf::from_str("extra/test.ftl").unwrap(),
                langs: vec![
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
                        }]
                        .into_iter()
                        .collect(),
                    },
                ],
            },
            MessageBundle {
                name: "main".to_string(),
                path: PathBuf::from_str("main.ftl").unwrap(),
                langs: vec![
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
                        }]
                        .into_iter()
                        .collect(),
                    },
                ],
            },
        ];

        let actual = super::create_message_bundles(resources.clone()).unwrap();

        assert_eq!(expected, actual);
    }
}
