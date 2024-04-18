#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate rustler;

mod inkjet_adapter;
mod types;

use std::cell::RefCell;

use ammonia::clean;
use comrak::{
    markdown_to_html, markdown_to_html_with_plugins,
    nodes::{Ast, AstNode, LineColumn, NodeHeading, NodeValue},
    Arena, ComrakPlugins, ExtensionOptions, ListStyleType, Options, ParseOptions, RenderOptions,
};
use inkjet_adapter::InkjetAdapter;
use rustler::{Env, NifResult, NifTuple, NifUntaggedEnum, Term};
use types::options::*;

rustler::init!(
    "Elixir.MDEx.Native",
    [parse_document, ast_to_html, to_html, to_html_with_options]
);

#[rustler::nif(schedule = "DirtyCpu")]
fn to_html(md: &str) -> String {
    let inkjet_adapter = InkjetAdapter::new("onedark");
    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&inkjet_adapter);
    markdown_to_html_with_plugins(md, &Options::default(), &plugins)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn to_html_with_options<'a>(env: Env<'a>, md: &str, options: ExOptions) -> NifResult<Term<'a>> {
    let comrak_options = comrak::Options {
        extension: extension_options_from_ex_options(&options),
        parse: parse_options_from_ex_options(&options),
        render: render_options_from_ex_options(&options),
    };
    match options.features.syntax_highlight_theme {
        Some(theme) => {
            let inkjet_adapter = InkjetAdapter::new(&theme);
            let mut plugins = ComrakPlugins::default();
            plugins.render.codefence_syntax_highlighter = Some(&inkjet_adapter);
            let unsafe_html = markdown_to_html_with_plugins(md, &comrak_options, &plugins);
            render(env, unsafe_html, options.features.sanitize)
        }
        None => {
            let unsafe_html = markdown_to_html(md, &comrak_options);
            render(env, unsafe_html, options.features.sanitize)
        }
    }
}

fn extension_options_from_ex_options(options: &ExOptions) -> ExtensionOptions {
    let mut extension_options = ExtensionOptions::default();

    extension_options.strikethrough = options.extension.strikethrough;
    extension_options.tagfilter = options.extension.tagfilter;
    extension_options.table = options.extension.table;
    extension_options.autolink = options.extension.autolink;
    extension_options.tasklist = options.extension.tasklist;
    extension_options.superscript = options.extension.superscript;
    extension_options.header_ids = options.extension.header_ids.clone();
    extension_options.footnotes = options.extension.footnotes;
    extension_options.description_lists = options.extension.description_lists;
    extension_options.front_matter_delimiter = options.extension.front_matter_delimiter.clone();

    extension_options
}

fn parse_options_from_ex_options(options: &ExOptions) -> ParseOptions {
    let mut parse_options = ParseOptions::default();

    parse_options.smart = options.parse.smart;
    parse_options.default_info_string = options.parse.default_info_string.clone();
    parse_options.relaxed_tasklist_matching = options.parse.relaxed_tasklist_matching;
    parse_options.relaxed_autolinks = options.parse.relaxed_autolinks;

    parse_options
}

fn render_options_from_ex_options(options: &ExOptions) -> RenderOptions {
    let mut render_options = RenderOptions::default();

    render_options.hardbreaks = options.render.hardbreaks;
    render_options.github_pre_lang = options.render.github_pre_lang;
    render_options.full_info_string = options.render.full_info_string;
    render_options.width = options.render.width;
    render_options.unsafe_ = options.render.unsafe_;
    render_options.escape = options.render.escape;
    render_options.list_style = ListStyleType::from(options.render.list_style.clone());
    render_options.sourcepos = options.render.sourcepos;

    render_options
}

fn render(env: Env, unsafe_html: String, sanitize: bool) -> NifResult<Term> {
    let html = match sanitize {
        true => clean(&unsafe_html),
        false => unsafe_html,
    };

    rustler::serde::to_term(env, html).map_err(|err| err.into())
}

#[derive(Debug, Clone, PartialEq, NifUntaggedEnum)]
enum ExNode {
    Document(ExNodeTuple),
    Heading(ExNodeTuple),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, NifTuple)]
struct ExNodeTuple {
    name: String,
    attrs: Vec<ExNodeAttr>,
    children: Vec<ExNode>,
}

#[derive(Debug, Clone, PartialEq, NifTuple)]
struct ExNodeAttr {
    name: String,
    value: ExNodeAttrValue,
}

#[derive(Debug, Clone, PartialEq, NifUntaggedEnum)]
enum ExNodeAttrValue {
    Level(u8),
    Setext(bool),
}

impl ExNode {
    // TODO: options
    pub fn parse_document(md: &str) -> Self {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, md, &comrak::ComrakOptions::default());
        Self::from(root)
    }

    // TODO: options
    pub fn format_document(&self) -> String {
        let arena = Arena::new();

        if let ExNode::Document(ExNodeTuple {
            name,
            attrs,
            children,
        }) = self
        {
            let mut output = vec![];
            let ast_node = self.to_ast_nodee(
                &arena,
                ExNode::Document(ExNodeTuple {
                    name: name.clone(),
                    attrs: attrs.to_vec(),
                    children: children.to_vec(),
                }),
            );

            comrak::html::format_document(ast_node, &Options::default(), &mut output).unwrap();
            String::from_utf8(output).unwrap()
        } else {
            // TODO: return Result
            panic!("Expected `document` node in AST")
        }
    }

    fn ast<'a>(&self, arena: &'a Arena<AstNode<'a>>, node_value: NodeValue) -> &AstNode<'a> {
        arena.alloc(AstNode::new(RefCell::new(Ast::new(
            node_value,
            LineColumn { line: 0, column: 0 },
        ))))
    }

    fn to_ast_nodee<'a>(
        &'a self,
        arena: &'a Arena<AstNode<'a>>,
        exnode: ExNode,
    ) -> &'a AstNode<'a> {
        let build = |node_value: NodeValue, children: Vec<ExNode>| {
            let parent = self.ast(arena, node_value);

            for child in children {
                let ast_child = self.to_ast_nodee(&arena, child);
                parent.append(ast_child);
            }

            parent
        };

        match exnode {
            ExNode::Document(ExNodeTuple {
                name,
                attrs,
                children,
            }) => build(NodeValue::Document, children),

            ExNode::Heading(ExNodeTuple {
                name,
                attrs,
                children,
            }) => build(
                NodeValue::Heading(NodeHeading {
                    level: 1,
                    setext: false,
                }),
                children,
            ),

            ExNode::Text(text) => build(NodeValue::Text(text.to_owned()), vec![]),

        }
    }
}

impl<'a> From<&'a AstNode<'a>> for ExNode {
    fn from(ast_node: &'a AstNode<'a>) -> Self {
        let children = ast_node
            .children()
            .map(|child| Self::from(child))
            .collect::<Vec<_>>();

        let node_value = &ast_node.data.borrow().value;

        match node_value {
            NodeValue::Document => Self::Document(ExNodeTuple {
                name: "document".to_string(),
                attrs: vec![],
                children,
            }),
            NodeValue::Heading(ref heading) => Self::Heading(ExNodeTuple {
                name: "heading".to_string(),
                attrs: vec![
                    ExNodeAttr {
                        name: "level".to_string(),
                        value: ExNodeAttrValue::Level(heading.level),
                    },
                    ExNodeAttr {
                        name: "setext".to_string(),
                        value: ExNodeAttrValue::Setext(heading.setext),
                    },
                ],
                children,
            }),
            NodeValue::Text(ref text) => Self::Text(text.to_string()),
            _ => todo!(),
        }
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn parse_document<'a>(env: Env<'a>, md: &str) -> ExNode {
    let node = ExNode::parse_document(md);
    println!("parsed: {:?}", node);
    node
}

#[rustler::nif(schedule = "DirtyCpu")]
fn ast_to_html(ast: ExNode) -> String {
    println!("ast...: {:?}", ast);
    ExNode::format_document(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_parse_document() {
    //     let parsed = ExNode::Document(
    //         vec![],
    //         vec![
    //             ExNode::Heading(
    //                 vec![ExAttr::Level(1)],
    //                 vec![ExNode::Text("header".to_string())],
    //             ),
    //             ExNode::Paragraph(
    //                 vec![],
    //                 vec![ExNode::Emph(
    //                     vec![],
    //                     vec![ExNode::Text("hello".to_string())],
    //                 )],
    //             ),
    //         ],
    //     );

    //     assert_eq!(ExNode::parse_document("# header\n*hello*"), parsed);
    // }

    // #[test]
    // fn format_document_from_exnode() {
    //     let exnode = ExNode::parse_document("# header");
    //     let astnode = exnode.format_document();
    //     println!("{:?}", astnode);
    // }
}
