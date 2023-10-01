use ruulang_core::{
    config::config::RuuLangConfig,
    parser::{
        parse_location::{
            Context, Descendable, DescentContext, Identifier, IdentifierKind, Parsed,
        },
        ruulang_ast::{Attribute, Fragment, Grant},
        schema_ast::{Entity, Relationship},
    },
    utils::error::{RuuLangError, TypecheckError},
    workspace::workspace::Workspace,
};
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};
use tower_lsp::{
    jsonrpc,
    lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
        Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams,
        InitializeResult, InitializedParams, MarkupContent, MarkupKind, MessageType,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    Client, LanguageServer,
};

use crate::utils::{get_line_prefix_sum, location_pair_to_range, position_to_location};

pub struct RuuLangServer {
    client: Client,

    workspaces: Mutex<Vec<Workspace>>,
}

impl RuuLangServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspaces: Mutex::new(vec![]),
        }
    }

    async fn show_diagnostics(&self, uri: Url, contents: &String, errors: &Vec<RuuLangError>) {
        let mut diagnostics = vec![];

        for error in errors {
            match error {
                RuuLangError::TypecheckError(TypecheckError::GeneralError(data)) => {
                    if data.loc.is_none() {
                        continue;
                    }

                    let loc = data.loc.unwrap();

                    diagnostics.push(Diagnostic::new(
                        location_pair_to_range(&contents, loc.0 as u32, loc.1 as u32),
                        Some(DiagnosticSeverity::ERROR),
                        None,
                        None,
                        data.data.clone(),
                        None,
                        None,
                    ))
                }
                RuuLangError::RuuLangParseError(location) => diagnostics.push(Diagnostic::new(
                    location_pair_to_range(&contents, *location as u32, *location as u32 + 1),
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    None,
                    "Parse error".to_string(),
                    None,
                    None,
                )),
                _ => {}
            }
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn workspace_for_file(&self, file_uri: &Url) -> Option<MappedMutexGuard<'_, Workspace>> {
        let file_name = file_uri.to_file_path().unwrap();

        let workspace_index = self
            .workspaces
            .lock()
            .await
            .iter()
            .position(|workspace| workspace.contains_file(&file_name))?;

        let workspaces = self.workspaces.lock().await;
        let result = MutexGuard::map(workspaces, |workspaces| &mut workspaces[workspace_index]);

        Some(result)
    }

    async fn patch_and_notify(&self, file_uri: &Url, contents: &String, allow_create: bool) {
        let file_path = file_uri.to_file_path();
        if file_path.is_err() {
            // We're just editing an unsaved buffer
            return;
        }

        let file_name = file_path.unwrap();
        let maybe_workspace = self.workspace_for_file(&file_uri).await;

        if let Some(mut workspace) = maybe_workspace {
            if !allow_create && !workspace.contains_file(&file_name) {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Skipping file: {:#?}", &file_name),
                    )
                    .await;

                return;
            }

            let success = workspace.patch_file(&file_name, contents).await;
            match success {
                Ok(_) => {
                    let contents = &contents;
                    let errors = workspace.typecheck_file(&file_name).await;

                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Typechecking {:#?}: {:#?}", &file_name, &errors),
                        )
                        .await;

                    self.show_diagnostics(file_uri.clone(), &contents, &errors)
                        .await;
                }
                Err(err) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("{:#?}", err))
                        .await;
                }
            }
        }
    }

    fn serialize_entity(&self, entity: &Parsed<Entity>) -> String {
        let mut result = String::new();
        result.push_str(format!("```ruulang\nentity {}\n```\n\n", entity.data.name).as_str());

        if let Some(docstring) = &entity.docstring {
            result.push_str(&docstring);
        }

        result
    }

    fn serialize_fragment(&self, entity: &Parsed<Entity>, fragment: &Parsed<Fragment>) -> String {
        let mut result = String::new();
        result.push_str(
            format!(
                "```ruulang\nfragment {} for {}\n```\n\n",
                fragment.data.name, entity.data.name,
            )
            .as_str(),
        );

        if let Some(docstring) = &fragment.docstring {
            result.push_str(&docstring);
        }

        result
    }

    fn serialize_relationship(
        &self,
        from_entity: &Parsed<Entity>,
        relationship: &Parsed<Relationship>,
        to_entity: &Parsed<Entity>,
    ) -> String {
        let mut result = String::new();

        result.push_str(
            format!(
                "```ruulang\nentity {} {{ {} -> {}; }}\n```\n\n",
                &from_entity.data.name, &relationship.relationship_name, &to_entity.data.name
            )
            .as_str(),
        );

        if let Some(docstring) = &relationship.docstring {
            result.push_str(&docstring);
        }

        result
    }

    fn serialize_grant(&self, entity: &Parsed<Entity>, grant: &Parsed<Grant>) -> String {
        let mut result = String::new();

        result.push_str(
            format!(
                "```ruulang\nentity {} {{ {}; }}\n```\n\n",
                &entity.data.name, &grant
            )
            .as_str(),
        );

        if let Some(docstring) = &grant.docstring {
            result.push_str(&docstring);
        }

        result
    }

    fn serialize_attribute(
        &self,
        rule: &Parsed<Relationship>,
        attribute: &Parsed<Attribute>,
    ) -> String {
        let mut result = String::new();

        result.push_str(
            format!(
                "```ruulang\n {}:{}\n```\n\n",
                &rule.data.relationship_name, &attribute.data.name
            )
            .as_str(),
        );

        if let Some(docstring) = &attribute.docstring {
            result.push_str(&docstring);
        }

        result
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for RuuLangServer {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        let mut workspaces = vec![];

        if let Some(folders) = params.workspace_folders {
            for folder in folders {
                let root_dir = folder.uri.to_file_path().unwrap();
                let file_path = root_dir.join("ruu.toml");
                let config = RuuLangConfig::load(&file_path, &root_dir).await;

                if let Err(err) = config {
                    self.client
                        .log_message(MessageType::ERROR, format!("{:#?}", err))
                        .await;
                    continue;
                }

                let mut workspace = Workspace::new(config.unwrap(), root_dir);
                workspace.reload().await;
                workspaces.push(workspace)
            }
        }

        self.workspaces.lock().await.extend(workspaces);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),

                ..Default::default()
            },

            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let file_uri = &params.text_document.uri;
        let contents = &params.text_document.text;

        if params.text_document.language_id != "ruulang" {
            return;
        }

        self.patch_and_notify(file_uri, contents, true).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let file_uri = &params.text_document.uri;
        let contents = &params.content_changes[0].text;

        self.patch_and_notify(file_uri, contents, false).await;
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let file_uri = params.text_document_position_params.text_document.uri;
        let file_name = file_uri.to_file_path().unwrap();
        let maybe_workspace = self.workspace_for_file(&file_uri).await;

        let workspace = match maybe_workspace {
            Some(workspace) => workspace,
            None => return Ok(None),
        };

        let contents = match workspace.resolve_schema(&file_name) {
            Some(data) => data,
            None => return Ok(None),
        };

        let schema = match &contents.data {
            Ok(schema) => schema,
            Err(_) => return Ok(None),
        };

        let file_contents = match workspace.resolve_file(&file_name) {
            Some(data) => data,
            None => return Ok(None),
        };

        let location = position_to_location(
            &get_line_prefix_sum(file_contents),
            &params.text_document_position_params.position,
        ) as usize;

        let descension = schema.descend_at((location, location));

        if let Some(stack) = descension {
            let mut entities: Vec<&Entity> = vec![];
            let mut rels: Vec<&Relationship> = vec![];
            let mut final_result: Option<String> = None;

            self.client
                .log_message(MessageType::INFO, format!("Stack {:#?}", &stack))
                .await;

            for ctx in stack {
                match (&entities.last(), rels.last(), &ctx) {
                    (
                        None,
                        _,
                        DescentContext {
                            context: Context::Entrypoint(entrypoint),
                            ..
                        },
                    ) => {
                        let entity_name = &entrypoint.entrypoint;
                        let Some(found_entity) = workspace.entity_by_name(&entity_name)
                        else { continue; };

                        entities.push(&found_entity.data.data);
                        final_result = Some(self.serialize_entity(&found_entity.data));
                    }

                    (
                        None,
                        _,
                        DescentContext {
                            context: Context::Entity(entity),
                            ..
                        },
                    ) => {
                        let entity_name = &entity.name;

                        let Some(found_entity) = workspace.entity_by_name(&entity_name)
                        else { continue; };

                        entities.push(&found_entity.data.data);
                        final_result = Some(self.serialize_entity(&found_entity.data));
                    }

                    (
                        None,
                        _,
                        DescentContext {
                            context: Context::Fragment(found_fragment),
                            ..
                        },
                    ) => {
                        let entity_name = &found_fragment.for_entity.data;
                        let fragment_name = &found_fragment.name;

                        let Some(found_entity) = workspace.entity_by_name(&entity_name)
                        else { continue; };

                        let Some(fragment) = workspace.fragment_by_name_and_entity(&fragment_name, &entity_name)
                        else { continue; };

                        entities.push(&found_entity.data.data);
                        final_result = Some(self.serialize_fragment(&found_entity.data, &fragment));
                    }

                    (
                        Some(entity),
                        _,
                        DescentContext {
                            context: Context::Rule(rule),
                            ..
                        },
                    ) => {
                        let Some(from_entity) = workspace.entity_by_name(&entity.name)
                        else { continue; };

                        let Some(relationship_object) = entity
                            .relationships
                            .iter()
                            .find(|x| x.data.relationship_name.data == rule.relationship.data)
                        else { continue };

                        let Some(next_entity) = workspace.entity_by_name(&relationship_object.data.entity_name)
                        else { continue };

                        final_result = Some(self.serialize_relationship(
                            &from_entity.data,
                            &relationship_object,
                            &next_entity.data,
                        ));

                        entities.push(&next_entity.data.data);
                        rels.push(&relationship_object);
                    }

                    (
                        _,
                        Some(rel),
                        DescentContext {
                            context: Context::Attribute(attr),
                            ..
                        },
                    ) => {
                        let parent_entity = entities[entities.len() - 2];

                        self.client
                            .log_message(
                                MessageType::INFO,
                                format!(
                                    "{} -({}:{})-> ?",
                                    parent_entity.name, rel.relationship_name, attr.name
                                ),
                            )
                            .await;

                        let Some(relationship_object) = parent_entity
                            .relationships
                            .iter()
                            .find(|x| x.data.relationship_name.data == rel.relationship_name.data)
                        else { continue };

                        let Some(found_attr) = relationship_object.attributes.iter().find(|x| x.name.data == attr.name.data)
                        else { continue };

                        final_result =
                            Some(self.serialize_attribute(relationship_object, found_attr));
                    }

                    (
                        Some(entity),
                        _,
                        DescentContext {
                            context: Context::Grant(g),
                            ..
                        },
                    ) => {
                        let Some(found_entity) = workspace.entity_by_name(&entity.name)
                        else { continue; };

                        let Some(found_grant) = entity.grants.iter().find(|x| x.data == ***g)
                        else { continue; };

                        final_result = Some(self.serialize_grant(&found_entity.data, &found_grant));
                    }

                    (
                        Some(entity),
                        _,
                        DescentContext {
                            context: Context::Identifier(id),
                            ..
                        },
                    ) => match **id {
                        Identifier {
                            value: fragment,
                            kind: IdentifierKind::Fragment,
                        } => {
                            let entity_name = &entity.name;
                            let fragment_name = &fragment;

                            let Some(found_entity) = workspace.entity_by_name(&entity_name)
                            else { continue; };

                            let Some(fragment) = workspace.fragment_by_name_and_entity(&fragment_name, &entity_name)
                            else { continue; };

                            final_result =
                                Some(self.serialize_fragment(&found_entity.data, &fragment));
                        }

                        Identifier {
                            value: entity_name,
                            kind: IdentifierKind::Entity,
                        } => {
                            let Some(found_entity) = workspace.entity_by_name(&entity_name)
                            else { continue; };

                            final_result = Some(self.serialize_entity(&found_entity.data));
                        }

                        _ => {}
                    },

                    _ => {}
                }
            }

            if let Some(result) = final_result {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: result,
                    }),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }
}
