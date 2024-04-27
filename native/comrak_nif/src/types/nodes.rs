use crate::types::options::*;
use comrak::{
    nodes::{Ast, AstNode, LineColumn, NodeHeading, NodeList, NodeValue},
    Arena, Options,
};
use rustler::{
    types::tuple::get_tuple, Binary, Decoder, Encoder, Env, NifResult, NifTuple, NifUntaggedEnum,
    Term,
};
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub struct ExNode {
    pub data: ExNodeData,
    pub children: ExNodeChildren,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExNodeData {
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
pub struct ExNodeList {
    pub list_type: ExListType,
    pub marker_offset: usize,
    pub padding: usize,
    pub start: usize,
    pub delimiter: ExListDelimType,
    pub bullet_char: u8,
    pub tight: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExListType {
    Bullet,
    Ordered,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExListDelimType {
    Period,
    Paren,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeDescriptionItem {
    pub marker_offset: usize,
    pub padding: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeCodeBlock {
    pub fenced: bool,
    pub fence_char: u8,
    pub fence_length: usize,
    pub fence_offset: usize,
    pub info: String,
    pub literal: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeHtmlBlock {
    pub block_type: u8,
    pub literal: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeHeading {
    pub level: u8,
    pub setext: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeFootnoteDefinition {
    pub name: String,
    pub total_references: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeTable {
    pub alignments: Vec<ExTableAlignment>,
    pub num_columns: usize,
    pub num_rows: usize,
    pub num_nonempty_cells: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExTableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeCode {
    pub num_backticks: usize,
    pub literal: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeLink {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeFootnoteReference {
    pub name: String,
    pub ref_num: u32,
    pub ix: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeMath {
    pub dollar_math: bool,
    pub display_math: bool,
    pub literal: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExNodeMultilineBlockQuote {
    pub fence_length: usize,
    pub fence_offset: usize,
}

pub type ExNodeAttrs = Vec<ExNodeAttr>;
pub type ExNodeChildren = Vec<ExNode>;

#[derive(Debug, Clone, PartialEq, NifTuple)]
pub struct ExNodeAttr(pub String, pub ExNodeAttrValue);

#[derive(Debug, Clone, PartialEq, NifUntaggedEnum)]
pub enum ExNodeAttrValue {
    U8(u8),
    U32(u32),
    Usize(usize),
    Bool(bool),
    Text(String),
    List(Vec<String>),
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

// decoding
impl<'a> Decoder<'a> for ExNode {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let node = ExNode::decode_term(term);
        Ok(node)
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

            // description list
            // description item
            // description details
            NodeValue::CodeBlock(ref code_block) => Self {
                data: ExNodeData::CodeBlock(ExNodeCodeBlock {
                    fenced: code_block.fenced,
                    fence_char: code_block.fence_char,
                    fence_length: code_block.fence_length,
                    fence_offset: code_block.fence_offset,
                    info: code_block.info.to_string(),
                    literal: code_block.literal.to_string(),
                }),
                children,
            },

            NodeValue::HtmlBlock(ref html_block) => Self {
                data: ExNodeData::HtmlBlock(ExNodeHtmlBlock {
                    block_type: html_block.block_type,
                    literal: html_block.literal.to_string(),
                }),
                children,
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

            // footnote definition
            NodeValue::Table(ref table) => Self {
                data: ExNodeData::Table(ExNodeTable {
                    // FIXME: resolve alignments list
                    alignments: vec![ExTableAlignment::Center],
                    num_columns: table.num_columns,
                    num_rows: table.num_rows,
                    num_nonempty_cells: table.num_nonempty_cells,
                }),
                children,
            },

            NodeValue::TableRow(ref header) => Self {
                data: ExNodeData::TableRow(*header),
                children,
            },

            NodeValue::TableCell => Self {
                data: ExNodeData::TableCell,
                children,
            },

            NodeValue::Text(ref text) => Self {
                data: ExNodeData::Text(text.to_string()),
                children: vec![],
            },

            NodeValue::TaskItem(ref symbol) => Self {
                data: ExNodeData::TaskItem(*symbol),
                children,
            },

            NodeValue::SoftBreak => Self {
                data: ExNodeData::SoftBreak,
                children,
            },
            NodeValue::LineBreak => Self {
                data: ExNodeData::LineBreak,
                children,
            },

            NodeValue::Code(ref code) => Self {
                data: ExNodeData::Code(ExNodeCode {
                    num_backticks: code.num_backticks,
                    literal: code.literal.to_string(),
                }),
                children,
            },

            // html inline
            NodeValue::Emph => Self {
                data: ExNodeData::Emph,
                children,
            },

            NodeValue::Strong => Self {
                data: ExNodeData::Strong,
                children,
            },

            NodeValue::Strikethrough => Self {
                data: ExNodeData::Strikethrough,
                children,
            },

            NodeValue::Superscript => Self {
                data: ExNodeData::Superscript,
                children,
            },

            // link
            NodeValue::Link(ref link) => Self {
                data: ExNodeData::Link(ExNodeLink {
                    url: link.url.to_string(),
                    title: link.title.to_string(),
                }),
                children,
            },

            // image
            NodeValue::Image(ref link) => Self {
                data: ExNodeData::Image(ExNodeLink {
                    url: link.url.to_string(),
                    title: link.title.to_string(),
                }),
                children,
            },

            // footnode reference
            // shortcode
            // math
            // multiline blockquote
            NodeValue::Escaped => Self {
                data: ExNodeData::Escaped,
                children,
            },

            _ => todo!("exnode from astnode"),
        }
    }
}

// encoding
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
            ExNode {
                data: ExNodeData::DescriptionList,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("description_list".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // description item
            ExNode {
                data: ExNodeData::DescriptionItem(node_description_item),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "description_item".to_string(),
                    node_description_item.encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // description term
            ExNode {
                data: ExNodeData::DescriptionTerm,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("description_term".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // description details
            ExNode {
                data: ExNodeData::DescriptionDetails,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("description_details".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

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
            ExNode {
                data: ExNodeData::HtmlBlock(html_block),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "html_block".to_string(),
                    html_block.encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

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
            ExNode {
                data: ExNodeData::FootnoteDefinition(node_footnote_definition),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "footnote_definition".to_string(),
                    node_footnote_definition.encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // table
            ExNode {
                data: ExNodeData::Table(node_table),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "table".to_string(),
                    node_table.encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // table row
            ExNode {
                data: ExNodeData::TableRow(header),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "table_row".to_string(),
                    vec![ExNodeAttr(
                        "header".to_string(),
                        ExNodeAttrValue::Text(header.to_string()),
                    )]
                    .encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // table cell
            ExNode {
                data: ExNodeData::TableCell,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("table_cell".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // text
            ExNode {
                data: ExNodeData::Text(text),
                children,
            } => text.encode(env),

            // task item
            ExNode {
                data: ExNodeData::TaskItem(symbol),
                children,
            } => {
                let symbol = symbol.unwrap_or(' ');

                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "task_item".to_string(),
                    vec![ExNodeAttr(
                        "symbol".to_string(),
                        ExNodeAttrValue::Text(symbol.to_string()),
                    )]
                    .encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // soft break
            ExNode {
                data: ExNodeData::SoftBreak,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("soft_break".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // line break
            ExNode {
                data: ExNodeData::LineBreak,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("line_break".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // code
            ExNode {
                data: ExNodeData::Code(code),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) =
                    ("code".to_string(), code.encode(env), children.to_vec());
                doc.encode(env)
            }

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
            ExNode {
                data: ExNodeData::Strikethrough,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("strikethrough".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // superscript
            ExNode {
                data: ExNodeData::Superscript,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("superscript".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

            // link
            ExNode {
                data: ExNodeData::Link(node_link),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) =
                    ("link".to_string(), node_link.encode(env), children.to_vec());
                doc.encode(env)
            }

            // image
            ExNode {
                data: ExNodeData::Image(node_link),
                children,
            } => {
                let doc: (String, Term<'a>, ExNodeChildren) = (
                    "image".to_string(),
                    node_link.encode(env),
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // footnote reference

            // short code

            // math

            // multiline block quote

            // escaped
            ExNode {
                data: ExNodeData::Escaped,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("escaped".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }

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
        // FIXME: extract into fn and handle error
        let c = std::char::from_u32(self.fence_char as u32).unwrap();

        vec![
            ExNodeAttr("fenced".to_string(), ExNodeAttrValue::Bool(self.fenced)),
            ExNodeAttr(
                "fence_char".to_string(),
                ExNodeAttrValue::Text(c.to_string()),
            ),
            ExNodeAttr(
                "fence_length".to_string(),
                ExNodeAttrValue::Usize(self.fence_length),
            ),
            ExNodeAttr(
                "fence_offset".to_string(),
                ExNodeAttrValue::Usize(self.fence_offset),
            ),
            ExNodeAttr(
                "info".to_string(),
                ExNodeAttrValue::Text(self.info.to_string()),
            ),
            ExNodeAttr(
                "literal".to_string(),
                ExNodeAttrValue::Text(self.literal.to_string()),
            ),
        ]
        .encode(env)
    }
}

impl Encoder for &ExNodeHtmlBlock {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr(
                "block_type".to_string(),
                ExNodeAttrValue::U8(self.block_type),
            ),
            ExNodeAttr(
                "literal".to_string(),
                ExNodeAttrValue::Text(self.literal.to_string()),
            ),
        ]
        .encode(env)
    }
}

impl Encoder for &ExNodeFootnoteDefinition {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr(
                "name".to_string(),
                ExNodeAttrValue::Text(self.name.to_string()),
            ),
            ExNodeAttr(
                "total_references".to_string(),
                ExNodeAttrValue::U32(self.total_references),
            ),
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

impl Encoder for &ExNodeDescriptionItem {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr(
                "marker_offset".to_string(),
                ExNodeAttrValue::Usize(self.marker_offset),
            ),
            ExNodeAttr("padding".to_string(), ExNodeAttrValue::Usize(self.padding)),
        ]
        .encode(env)
    }
}

impl Encoder for &ExNodeTable {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr(
                "alignments".to_string(),
                // FIXME: resolve alignment list
                ExNodeAttrValue::List(vec!["center".to_string()]),
            ),
            ExNodeAttr(
                "num_columns".to_string(),
                ExNodeAttrValue::Usize(self.num_columns),
            ),
            ExNodeAttr(
                "num_rows".to_string(),
                ExNodeAttrValue::Usize(self.num_rows),
            ),
            ExNodeAttr(
                "num_nomempty_cells".to_string(),
                ExNodeAttrValue::Usize(self.num_nonempty_cells),
            ),
        ]
        .encode(env)
    }
}

impl Encoder for &ExNodeLink {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr(
                "url".to_string(),
                ExNodeAttrValue::Text(self.url.to_string()),
            ),
            ExNodeAttr(
                "title".to_string(),
                ExNodeAttrValue::Text(self.title.to_string()),
            ),
        ]
        .encode(env)
    }
}

impl Encoder for &ExNodeCode {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        vec![
            ExNodeAttr(
                "num_backticks".to_string(),
                ExNodeAttrValue::Usize(self.num_backticks),
            ),
            ExNodeAttr(
                "literal".to_string(),
                ExNodeAttrValue::Text(self.literal.to_string()),
            ),
        ]
        .encode(env)
    }
}
