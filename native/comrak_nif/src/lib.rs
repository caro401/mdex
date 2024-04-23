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
    nodes::{Ast, AstNode, LineColumn, NodeHeading, NodeList, NodeValue},
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

#[derive(Debug, Clone, PartialEq)]
struct ExNode {
    data: ExNodeData,
    children: ExNodeChildren,
}

#[derive(Debug, Clone, PartialEq)]
enum ExNodeData {
    Document,
    FrontMatter(String),
    BlockQuote,
    List(ExNodeList),
    Item(ExNodeList),
    DescriptionList,
    DescriptionItem(ExNodeDescriptionItem),
    DescriptionTerm,
    DescriptionDetails,
    CodeBlock(ExNodeCodeBlock),
    HtmlBlock(ExNodeHtmlBlock),
    Paragraph,
    Heading(ExNodeHeading),
    ThematicBreak,
    FootnoteDefinition(ExNodeFootnoteDefinition),
    Table(ExNodeTable),
    TableRow(bool),
    TableCell,
    Text(String),
    TaskItem(Option<char>),
    SoftBreak,
    LineBreak,
    Code(ExNodeCode),
    HtmlInline(String),
    Emph,
    Strong,
    Strikethrough,
    Superscript,
    Link(ExNodeLink),
    Image(ExNodeLink),
    FootnoteReference(ExNodeFootnoteReference),
    ShortCode(String),
    Math(ExNodeMath),
    MultilineBlockQuote(ExNodeMultilineBlockQuote),
    Escaped,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeList {
    list_type: ExListType,
    marker_offset: usize,
    padding: usize,
    start: usize,
    delimiter: ExListDelimType,
    bullet_char: u8,
    tight: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum ExListType {
    Bullet,
    Ordered,
}

#[derive(Debug, Clone, PartialEq)]
enum ExListDelimType {
    Period,
    Paren,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeDescriptionItem {
    marker_offset: usize,
    padding: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeCodeBlock {
    fenced: bool,
    fence_char: u8,
    fence_length: usize,
    fence_offset: usize,
    info: String,
    literal: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeHtmlBlock {
    block_type: u8,
    literal: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeHeading {
    level: u8,
    setext: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeFootnoteDefinition {
    name: String,
    total_references: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeTable {
    alignments: Vec<ExTableAlignment>,
    num_columns: usize,
    num_rows: usize,
    num_nonempty_cells: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExTableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeCode {
    num_backticks: usize,
    literal: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeLink {
    url: String,
    title: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeFootnoteReference {
    name: String,
    ref_num: u32,
    ix: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeMath {
    dollar_math: bool,
    display_math: bool,
    literal: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ExNodeMultilineBlockQuote {
    fence_length: usize,
    fence_offset: usize,
}

type ExNodeAttrs = Vec<ExNodeAttr>;
type ExNodeChildren = Vec<ExNode>;

#[derive(Debug, Clone, PartialEq, NifTuple)]
struct ExNodeAttr(String, ExNodeAttrValue);

#[derive(Debug, Clone, PartialEq, NifUntaggedEnum)]
enum ExNodeAttrValue {
    U8(u8),
    Usize(usize),
    Bool(bool),
    Text(String),
}

impl ExNode {
    fn decode_term<'a>(term: Term<'a>) -> Self {
        if term.is_tuple() {
            let node: Vec<Term<'a>> = get_tuple(term).unwrap();

            match node.len() {
                3 => ExNode::decode_node(node),
                _ => todo!("decode term node.len != 3"),
            }
        } else if term.is_binary() {
            let text: String = term.decode().unwrap();
            ExNode {
                data: ExNodeData::Text(text),
                children: vec![],
            }
        } else {
            todo!("decode term else")
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
                    // FIXME: node heading attrs
                    level: 1,
                    setext: false,
                }),
                children,
            },
            "paragraph" => ExNode {
                data: ExNodeData::Paragraph,
                children,
            },
            "strong" => ExNode {
                data: ExNodeData::Strong,
                children,
            },
            "emph" => ExNode {
                data: ExNodeData::Emph,
                children,
            },
            "list" => ExNode {
                data: ExNodeData::List(ExNodeList {
                    // FIXME: node list attrs
                    list_type: ExListType::Bullet,
                    marker_offset: 2,
                    padding: 2,
                    start: 1,
                    delimiter: ExListDelimType::Period,
                    bullet_char: 45,
                    tight: true,
                }),
                children,
            },
            "item" => ExNode {
                data: ExNodeData::Item(ExNodeList {
                    // FIXME: node list attrs
                    list_type: ExListType::Bullet,
                    marker_offset: 2,
                    padding: 2,
                    start: 1,
                    delimiter: ExListDelimType::Period,
                    bullet_char: 45,
                    tight: true,
                }),
                children,
            },
            &_ => todo!("exnode decode_node"),
        }
    }

    pub fn parse_document(md: &str, options: ExOptions) -> Self {
        let comrak_options = comrak::Options {
            extension: extension_options_from_ex_options(&options),
            parse: parse_options_from_ex_options(&options),
            render: render_options_from_ex_options(&options),
        };
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, md, &comrak_options);
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
            let ast_node = self.to_ast_node(
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

    fn to_ast_node<'a>(&'a self, arena: &'a Arena<AstNode<'a>>, exnode: ExNode) -> &'a AstNode<'a> {
        let build = |node_value: NodeValue, children: Vec<ExNode>| {
            let parent = self.ast(arena, node_value);

            for child in children {
                let ast_child = self.to_ast_node(arena, child);
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
            ExNode {
                data: ExNodeData::Paragraph,
                children,
            } => build(NodeValue::Paragraph, children),
            ExNode {
                data: ExNodeData::Strong,
                children,
            } => build(NodeValue::Strong, children),
            ExNode {
                data: ExNodeData::Emph,
                children,
            } => build(NodeValue::Emph, children),
            ExNode {
                data: ExNodeData::List(ref node_list),
                children,
            } => build(
                NodeValue::List(NodeList {
                    // FIXME: node list attrs
                    list_type: comrak::nodes::ListType::Bullet,
                    marker_offset: node_list.marker_offset,
                    padding: node_list.padding,
                    start: node_list.start,
                    delimiter: comrak::nodes::ListDelimType::Period,
                    bullet_char: node_list.bullet_char,
                    tight: node_list.tight,
                }),
                children,
            ),
            ExNode {
                data: ExNodeData::Item(ref node_list),
                children,
            } => build(
                NodeValue::List(NodeList {
                    // FIXME: node list attrs
                    list_type: comrak::nodes::ListType::Bullet,
                    marker_offset: node_list.marker_offset,
                    padding: node_list.padding,
                    start: node_list.start,
                    delimiter: comrak::nodes::ListDelimType::Period,
                    bullet_char: node_list.bullet_char,
                    tight: node_list.tight,
                }),
                children,
            ),
            _ => todo!("exnode to_ast_node"),
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
        // println!("encode: {:?}", self);

        match self {
            // document
            ExNode {
                data: ExNodeData::Document,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("document".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // front matter
            ExNode {
                data: ExNodeData::FrontMatter(delimiter),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "front_matter".to_string(),
                    vec![ExNodeAttr(
                        "content".to_string(),
                        ExNodeAttrValue::Text(delimiter.to_string()),
                    )]
                    .encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // block quote
            ExNode {
                data: ExNodeData::BlockQuote,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("block_quote".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // list
            ExNode {
                data: ExNodeData::List(node_list),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) =
                    ("list".to_string(), node_list.encode(env), children.to_vec());
                doc.encode(env)
            }

            // item
            ExNode {
                data: ExNodeData::Item(node_list),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) =
                    ("item".to_string(), node_list.encode(env), children.to_vec());
                doc.encode(env)
            }

            // description list

            // description item

            // description term

            // description details

            // code block
            ExNode {
                data: ExNodeData::CodeBlock(code_block),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "code_block".to_string(),
                    code_block.encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // html block

            // paragraph
            ExNode {
                data: ExNodeData::Paragraph,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("paragraph".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // heading
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

            // thematic break
            ExNode {
                data: ExNodeData::ThematicBreak,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("thematic_break".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // footnote definition

            // table

            // table row

            // table cell

            // text
            ExNode {
                data: ExNodeData::Text(text),
                children,
            } => text.encode(env),

            // task item

            // soft break

            // line break

            // code

            // html inline

            // emph
            ExNode {
                data: ExNodeData::Emph,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("emph".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // strong
            ExNode {
                data: ExNodeData::Strong,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("strong".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // strikethrough

            // superscript

            // link

            // image

            // footnote reference

            // short code

            // math

            // multiline block quote

            // escaped
            _ => todo!("exnode encode"),
        }
    }
}

impl Encoder for &ExNodeHeading {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr("level".to_string(), ExNodeAttrValue::U8(self.level)),
            ExNodeAttr("setext".to_string(), ExNodeAttrValue::Bool(self.setext)),
        ]
        .encode(env)
    }
}

impl Encoder for &ExNodeCodeBlock {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr("fenced".to_string(), ExNodeAttrValue::Bool(self.fenced)),
            ExNodeAttr(
                "fence_char".to_string(),
                ExNodeAttrValue::U8(self.fence_char),
            ),
            ExNodeAttr(
                "fence_length".to_string(),
                ExNodeAttrValue::Usize(self.fence_length),
            ),
            ExNodeAttr(
                "fence_offset".to_string(),
                ExNodeAttrValue::Usize(self.fence_offset),
            ),
            ExNodeAttr("info".to_string(), ExNodeAttrValue::Text(self.info.to_string())),
            ExNodeAttr("literal".to_string(), ExNodeAttrValue::Text(self.literal.to_string())),
        ]
        .encode(env)
    }
}

impl ToString for ExListType {
    fn to_string(&self) -> String {
        match self {
            ExListType::Bullet => "bullet".to_string(),
            ExListType::Ordered => "ordered".to_string(),
        }
    }
}

impl ToString for ExListDelimType {
    fn to_string(&self) -> String {
        match self {
            ExListDelimType::Period => "period".to_string(),
            ExListDelimType::Paren => "paren".to_string(),
        }
    }
}

impl Encoder for &ExNodeList {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr(
                "list_type".to_string(),
                ExNodeAttrValue::Text(self.list_type.to_string()),
            ),
            ExNodeAttr(
                "marker_offset".to_string(),
                ExNodeAttrValue::Usize(self.marker_offset),
            ),
            ExNodeAttr("padding".to_string(), ExNodeAttrValue::Usize(self.padding)),
            ExNodeAttr("start".to_string(), ExNodeAttrValue::Usize(self.start)),
            ExNodeAttr(
                "delimiter".to_string(),
                ExNodeAttrValue::Text(self.delimiter.to_string()),
            ),
            ExNodeAttr(
                "bullet_char".to_string(),
                ExNodeAttrValue::U8(self.bullet_char),
            ),
            ExNodeAttr("tight".to_string(), ExNodeAttrValue::Bool(self.tight)),
        ]
        .encode(env)
    }
}

impl<'a> From<&'a AstNode<'a>> for ExNode {
    fn from(ast_node: &'a AstNode<'a>) -> Self {
        let children = ast_node.children().map(Self::from).collect::<Vec<_>>();
        let node_value = &ast_node.data.borrow().value;

        println!("node_value: {:?}", node_value);

        match node_value {
            NodeValue::Document => Self {
                data: ExNodeData::Document,
                children,
            },

            NodeValue::FrontMatter(ref content) => Self {
                data: ExNodeData::FrontMatter(content.to_string()),
                children,
            },

            NodeValue::BlockQuote => Self {
                data: ExNodeData::BlockQuote,
                children,
            },

            // FIXME: list attrs
            NodeValue::List(ref node_list) => Self {
                data: ExNodeData::List(ExNodeList {
                    // FIXME: node list attrs
                    list_type: ExListType::Bullet,
                    marker_offset: node_list.marker_offset,
                    padding: node_list.padding,
                    start: node_list.start,
                    delimiter: ExListDelimType::Period,
                    bullet_char: node_list.bullet_char,
                    tight: node_list.tight,
                }),
                children,
            },

            // FIXME: item attrs
            NodeValue::Item(ref node_list) => Self {
                data: ExNodeData::Item(ExNodeList {
                    // FIXME: node list attrs
                    list_type: ExListType::Bullet,
                    marker_offset: node_list.marker_offset,
                    padding: node_list.padding,
                    start: node_list.start,
                    delimiter: ExListDelimType::Period,
                    bullet_char: node_list.bullet_char,
                    tight: node_list.tight,
                }),
                children,
            },

            NodeValue::CodeBlock(ref code_block) => Self {
                data: ExNodeData::CodeBlock(ExNodeCodeBlock {
                    fenced: code_block.fenced,
                    fence_char: code_block.fence_char,
                    fence_length: code_block.fence_length,
                    fence_offset: code_block.fence_offset,
                    info: code_block.info.to_string(),
                    literal: code_block.literal.to_string(),
                }),
                children
            },

            NodeValue::Paragraph => Self {
                data: ExNodeData::Paragraph,
                children,
            },

            NodeValue::Heading(ref heading) => Self {
                data: ExNodeData::Heading(ExNodeHeading {
                    level: heading.level,
                    setext: heading.setext,
                }),
                children,
            },

            NodeValue::ThematicBreak => Self {
                data: ExNodeData::ThematicBreak,
                children,
            },

            NodeValue::Text(ref text) => Self {
                data: ExNodeData::Text(text.to_string()),
                children: vec![],
            },

            NodeValue::Emph => Self {
                data: ExNodeData::Emph,
                children,
            },

            NodeValue::Strong => Self {
                data: ExNodeData::Strong,
                children,
            },

            _ => todo!("exnode from astnode"),
        }
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn parse_document(env: Env<'_>, md: &str, options: ExOptions) -> ExNode {
    ExNode::parse_document(md, options)
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
