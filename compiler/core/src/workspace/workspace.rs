use std::{collections::HashMap, path::PathBuf, str::from_utf8};

use crate::{
    codegen::{codegen::Codegen, python::PythonCodegen},
    config::config::RuuLangConfig,
    parser::{
        assembler::ParserAssemble,
        parse_location::Parsed,
        parser_constructs::ParserStatement,
        ruulang_ast::RuuLangFile,
        schema_ast::{Entity, RuuLangSchema},
    },
    typechecker::typechecker::Typechecker,
    utils::error::{Result, RuuLangError, TypecheckError},
};
use async_recursion::async_recursion;
use tokio::fs;

use crate::utils::with_origin::WithOrigin;

#[derive(Debug)]
pub struct Workspace {
    pub config: RuuLangConfig,
    pub working_dir: PathBuf,

    source_files: HashMap<PathBuf, String>,
    entities: Vec<WithOrigin<Parsed<Entity>>>,
    files: Vec<WithOrigin<Result<RuuLangFile>>>,
}

impl Workspace {
    pub fn new(config: RuuLangConfig, working_dir: PathBuf) -> Self {
        return Workspace {
            config,
            working_dir,

            source_files: HashMap::new(),
            entities: vec![],
            files: vec![],
        };
    }

    pub async fn reload(&mut self) {
        let files = self.gather().await;
        let file_data = self.read_all(&files).await;
        let (entities, files) = self.parse_all(&file_data);

        self.entities = entities;
        self.files = files;
        self.source_files = file_data;
    }

    pub async fn file_is_ruulang_source(&self, path: &PathBuf) -> bool {
        let extn = "ruulang";
        let path_extn = path.extension().unwrap_or_default();

        path_extn == extn
    }

    pub async fn compile_all(&self) -> Result<()> {
        let mut maybe_err = None;

        for schema in &self.files {
            let result = self.compile_one(schema).await;

            if let Err(err) = result {
                maybe_err = Some(err);
            }
        }

        if let Some(err) = maybe_err {
            Err(err)
        } else {
            Ok(())
        }
    }

    pub async fn typecheck_file(&self, path: &PathBuf) -> Vec<RuuLangError> {
        let schema = self.files.iter().find(|x| &x.origin == path);

        match schema {
            Some(schema) => {
                let typechecker = Typechecker::new(&self.entities, &self.files);

                match &schema.data {
                    Ok(data) => typechecker.validate_file(data),
                    Err(e) => vec![e.clone()],
                }
            }

            None => vec![RuuLangError::FileNotFound(format!(
                "File not found: {}",
                path.display()
            ))],
        }
    }

    pub async fn typecheck(&self) -> Result<()> {
        let typechecker = Typechecker::new(&self.entities, &self.files);

        let mut total_errors = 0;

        for schema in &self.files {
            match &schema.data {
                Ok(data) => {
                    let errors = typechecker.validate_file(data);
                    total_errors += errors.len();

                    if errors.len() > 0 {
                        println!("Errors in file: {}", schema.origin.display());
                    }

                    for error in errors {
                        if let RuuLangError::TypecheckError(TypecheckError::GeneralError(
                            general_error,
                        )) = error
                        {
                            println!("    [{:?}] {}", general_error.loc, general_error.data);
                        }
                    }
                }
                Err(e) => {
                    total_errors += 1;

                    println!("Error parsing file: {:?}", e);
                }
            }
        }

        println!("Finished Typechecking. {} error(s) found.", total_errors);
        Ok(())
    }

    pub async fn patch_file(&mut self, path: &PathBuf, contents: &String) -> Result<()> {
        self.source_files.insert(path.clone(), contents.clone());

        let result = self.parse_file(contents);
        let result_schema = result.as_ref().map(|(schema, _)| schema);

        self.entities.retain(|x| &x.origin != path);

        match result_schema {
            Ok(schemata) => {
                self.entities.extend(
                    schemata
                        .entities
                        .iter()
                        .map(|entity| WithOrigin::new(entity.clone(), path.clone())),
                );
            }
            Err(_) => {}
        };

        let result_file = result.map(|(_, file)| file);

        let index = self.files.iter().position(|x| &x.origin == path);
        match index {
            Some(idx) => {
                self.files[idx] = WithOrigin::new(result_file, path.clone());
            }
            None => {
                self.files.push(WithOrigin::new(result_file, path.clone()));
            }
        };

        Ok(())
    }

    pub fn file_name_iter(&self) -> impl Iterator<Item = &PathBuf> {
        self.source_files.keys()
    }

    fn parse_all(
        &self,
        file_data: &HashMap<PathBuf, String>,
    ) -> (
        Vec<WithOrigin<Parsed<Entity>>>,
        Vec<WithOrigin<Result<RuuLangFile>>>,
    ) {
        let mut entities = Vec::new();
        let mut files = Vec::new();

        for ((_, contents), (origin, _)) in file_data.iter().zip(file_data) {
            let parsed_contents = ParserStatement::parse(contents);
            match parsed_contents {
                Err(e) => files.push(WithOrigin::new(Err(RuuLangError::from(e)), origin.clone())),
                Ok(data) => {
                    let (schemata, rule) = data.assemble();

                    entities.extend(
                        schemata
                            .entities
                            .into_iter()
                            .map(|entity| WithOrigin::new(entity, origin.clone())),
                    );
                    files.push(WithOrigin::new(Ok(rule), origin.clone()));
                }
            };
        }

        (entities, files)
    }

    pub fn contains_file(&self, path: &PathBuf) -> bool {
        let root = self.config.workspace.root.as_ref().unwrap();
        path.starts_with(root)
    }

    pub fn resolve_file(&self, path: &PathBuf) -> Option<&String> {
        self.source_files.get(path)
    }

    pub fn resolve_schema(&self, path: &PathBuf) -> Option<&WithOrigin<Result<RuuLangFile>>> {
        self.files.iter().find(|x| &x.origin == path)
    }

    fn parse_file(&self, contents: &String) -> Result<(RuuLangSchema, RuuLangFile)> {
        let token_contents = ParserStatement::parse(from_utf8(contents.as_bytes()).unwrap())?;
        let (schemata, rule) = token_contents.assemble();

        Ok((schemata, rule))
    }

    async fn gather(&self) -> Vec<PathBuf> {
        #[async_recursion]
        async fn gather_at(root: &PathBuf, extn: &str) -> Vec<PathBuf> {
            let files = fs::read_dir(root).await;
            if let Err(..) = &files {
                return Vec::new();
            }

            let mut result = Vec::new();
            let mut file_list = files.unwrap();

            while let Ok(Some(next)) = file_list.next_entry().await {
                let path = next.path();
                let path_extn = path.extension().unwrap_or_default();

                if extn == path_extn {
                    result.push(path);
                } else if next.metadata().await.unwrap().is_dir() {
                    let mut subdir = gather_at(&path, extn).await;
                    result.append(&mut subdir)
                }
            }

            result
        }

        let root = &self.config.workspace.root;
        let start_dir = root.to_owned().unwrap_or(self.working_dir.clone());
        return gather_at(&start_dir, "ruu").await;
    }

    async fn read_all(&self, files: &Vec<PathBuf>) -> HashMap<PathBuf, String> {
        let mut result = HashMap::new();

        for file in files {
            match fs::read(file).await {
                Ok(bytes) => {
                    let contents = from_utf8(bytes.as_slice()).unwrap().to_string();
                    result.insert(file.clone(), contents);
                }
                Err(_) => {
                    // pass for now
                }
            }
        }

        result
    }

    async fn compile_one(&self, schema: &WithOrigin<Result<RuuLangFile>>) -> Result<()> {
        if self.config.json.as_ref().map_or(false, |x| x.enabled) {
            self.compile_one_json(schema).await?;
        }

        if self.config.python.as_ref().map_or(false, |x| x.enabled) {
            self.compile_one_python(schema).await?;
        }

        Ok(())
    }

    async fn compile_one_python(&self, schema: &WithOrigin<Result<RuuLangFile>>) -> Result<()> {
        let mut new_file = schema.origin.clone();
        new_file.set_extension("py");

        let file = match &schema.data {
            Ok(d) => d,
            Err(_) => return Ok(()),
        };

        let entities = &self.entities;

        let python = PythonCodegen::new(&schema.origin, &new_file, &self.config, entities, &file);
        let result = python.serialize_schema_and_file();

        fs::write(new_file, result).await?;

        Ok(())
    }

    async fn compile_one_json(&self, schema: &WithOrigin<Result<RuuLangFile>>) -> Result<()> {
        let mut new_file = schema.origin.clone();
        new_file.set_extension("json");

        let data = &schema.data.clone()?;
        let as_json = serde_json::to_string_pretty(&data).unwrap();
        fs::write(new_file, &as_json).await?;

        Ok(())
    }
}
