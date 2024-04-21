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
use rustler::{
    types::tuple::get_tuple, Binary, Decoder, Encoder, Env, NifResult, NifTuple, NifUntaggedEnum,
    Term,
};
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

type ExNodeAttrs = Vec<ExNodeAttr>;
type ExNodeChildren = Vec<ExNode>;

#[derive(Debug, Clone, PartialEq, NifTuple)]
struct ExNodeAttr(String, ExNodeAttrValue);

#[derive(Debug, Clone, PartialEq, NifUntaggedEnum)]
enum ExNodeAttrValue {
    Level(u8),
    Setext(bool),
}

#[derive(Debug, Clone, PartialEq)]
struct ExNode {
    data: ExNodeData,
    children: ExNodeChildren,
}

#[derive(Debug, Clone, PartialEq)]
enum ExNodeData {
    Document,
    Heading(ExNodeHeading),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeHeading {
    level: u8,
    setext: bool,
}

impl ExNode {
    fn decode_term<'a>(term: Term<'a>) -> Self {
        if term.is_tuple() {
            let node: Vec<Term<'a>> = get_tuple(term).unwrap();

            match node.len() {
                3 => ExNode::decode_node(node),
                _ => todo!(),
            }
        } else if term.is_binary() {
            let text: String = term.decode().unwrap();
            ExNode {
                data: ExNodeData::Text(text),
                children: vec![],
            }
        } else {
            todo!()
        }
    }

    fn decode_node<'a>(node: Vec<Term<'a>>) -> Self {
        // FIXME: find a better way to convert Term to String
        let name = node.first().unwrap();
        let name = Binary::from_term(*name).unwrap().as_slice();
        let name = String::from_utf8(name.to_vec()).unwrap();

        let children: Vec<Term<'a>> = node.get(2).unwrap().decode::<Vec<Term>>().unwrap();

        let children: Vec<_> = children
            .iter()
            .map(|child| ExNode::decode_term(*child))
            .collect();

        match name.as_str() {
            "document" => ExNode {
                data: ExNodeData::Document,
                children,
            },
            "heading" => ExNode {
                data: ExNodeData::Heading(ExNodeHeading {
                    level: 1,
                    setext: false,
                }),
                children,
            },
            &_ => todo!(),
        }
    }

    // TODO: options
    pub fn parse_document(md: &str) -> Self {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, md, &comrak::ComrakOptions::default());
        Self::from(root)
    }

    pub fn format_document(&self) -> String {
        let arena = Arena::new();

        if let ExNode {
            data: ExNodeData::Document,
            children,
        } = self
        {
            let mut output = vec![];
            let ast_node = self.to_ast_nodee(
                &arena,
                ExNode {
                    data: ExNodeData::Document,
                    children: children.to_vec(),
                },
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
                let ast_child = self.to_ast_nodee(arena, child);
                parent.append(ast_child);
            }

            parent
        };

        match exnode {
            ExNode {
                data: ExNodeData::Document,
                children,
            } => build(NodeValue::Document, children),

            ExNode {
                data: ExNodeData::Heading(ref heading),
                children,
            } => build(
                NodeValue::Heading(NodeHeading {
                    level: heading.level,
                    setext: heading.setext,
                }),
                children,
            ),

            ExNode {
                data: ExNodeData::Text(text),
                children,
            } => build(NodeValue::Text(text.to_owned()), vec![]),
        }
    }
}

impl<'a> Decoder<'a> for ExNode {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let node = ExNode::decode_term(term);
        Ok(node)
    }
}

impl Encoder for ExNode {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        match self {
            ExNode {
                data: ExNodeData::Document,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("document".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }
            ExNode {
                data: ExNodeData::Heading(heading),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "heading".to_string(),
                    heading.encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }
            ExNode {
                data: ExNodeData::Text(text),
                children,
            } => text.encode(env),
        }
    }
}

impl Encoder for &ExNodeHeading {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr("level".to_string(), ExNodeAttrValue::Level(self.level)),
            ExNodeAttr("setext".to_string(), ExNodeAttrValue::Setext(self.setext)),
        ]
        .encode(env)
    }
}

impl<'a> From<&'a AstNode<'a>> for ExNode {
    fn from(ast_node: &'a AstNode<'a>) -> Self {
        let children = ast_node.children().map(Self::from).collect::<Vec<_>>();

        let node_value = &ast_node.data.borrow().value;

        match node_value {
            NodeValue::Document => Self {
                data: ExNodeData::Document,
                children,
            },
            NodeValue::Heading(ref heading) => Self {
                data: ExNodeData::Heading(ExNodeHeading {
                    level: heading.level,
                    setext: heading.setext,
                }),
                children,
            },
            NodeValue::Text(ref text) => Self {
                data: ExNodeData::Text(text.to_string()),
                children: vec![],
            },
            _ => todo!(),
        }
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn parse_document(env: Env<'_>, md: &str) -> ExNode {
    ExNode::parse_document(md)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn ast_to_html(ast: ExNode) -> String {
    ast.format_document()
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
