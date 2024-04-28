use crate::types::options::*;
use comrak::{
    nodes::{Ast, AstNode, LineColumn, ListDelimType, ListType, NodeHeading, NodeList, NodeValue},
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
    ShortCode(ExNodeShortCode),
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
pub struct ExNodeShortCode {
    pub shortcode: String,
    pub emoji: String,
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

            NodeValue::List(ref node_list) => Self {
                data: ExNodeData::List(ExNodeList {
                    list_type: ExListType::from(node_list.list_type),
                    marker_offset: node_list.marker_offset,
                    padding: node_list.padding,
                    start: node_list.start,
                    delimiter: ExListDelimType::from(node_list.delimiter),
                    bullet_char: node_list.bullet_char,
                    tight: node_list.tight,
                }),
                children,
            },

            NodeValue::Item(ref node_list) => Self {
                data: ExNodeData::Item(ExNodeList {
                    list_type: ExListType::from(node_list.list_type),
                    marker_offset: node_list.marker_offset,
                    padding: node_list.padding,
                    start: node_list.start,
                    delimiter: ExListDelimType::from(node_list.delimiter),
                    bullet_char: node_list.bullet_char,
                    tight: node_list.tight,
                }),
                children,
            },

            NodeValue::DescriptionList => Self {
                data: ExNodeData::DescriptionList,
                children,
            },

            NodeValue::DescriptionItem(ref description_item) => Self {
                data: ExNodeData::DescriptionItem(ExNodeDescriptionItem {
                    marker_offset: description_item.marker_offset,
                    padding: description_item.padding,
                }),
                children,
            },

            NodeValue::DescriptionTerm => Self {
                data: ExNodeData::DescriptionTerm,
                children,
            },

            NodeValue::DescriptionDetails => Self {
                data: ExNodeData::DescriptionDetails,
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

            NodeValue::FootnoteDefinition(ref footnote_definition) => Self {
                data: ExNodeData::FootnoteDefinition(ExNodeFootnoteDefinition {
                    name: footnote_definition.name.to_string(),
                    total_references: footnote_definition.total_references,
                }),
                children,
            },

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

            NodeValue::HtmlInline(ref raw_html) => Self {
                data: ExNodeData::HtmlInline(raw_html.to_string()),
                children,
            },

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

            NodeValue::Link(ref link) => Self {
                data: ExNodeData::Link(ExNodeLink {
                    url: link.url.to_string(),
                    title: link.title.to_string(),
                }),
                children,
            },

            NodeValue::Image(ref link) => Self {
                data: ExNodeData::Image(ExNodeLink {
                    url: link.url.to_string(),
                    title: link.title.to_string(),
                }),
                children,
            },

            NodeValue::FootnoteReference(ref footnote_reference) => Self {
                data: ExNodeData::FootnoteReference(ExNodeFootnoteReference {
                    name: footnote_reference.name.to_string(),
                    ref_num: footnote_reference.ref_num,
                    ix: footnote_reference.ix,
                }),
                children,
            },

            NodeValue::ShortCode(ref short_code) => Self {
                data: ExNodeData::ShortCode(ExNodeShortCode {
                    shortcode: short_code.shortcode().to_string(),
                    emoji: short_code.emoji().to_string(),
                }),
                children,
            },

            NodeValue::Math(ref math) => Self {
                data: ExNodeData::Math(ExNodeMath {
                    dollar_math: math.dollar_math,
                    display_math: math.display_math,
                    literal: math.literal.to_string(),
                }),
                children,
            },

            NodeValue::MultilineBlockQuote(ref multiline_block_quote) => Self {
                data: ExNodeData::MultilineBlockQuote(ExNodeMultilineBlockQuote {
                    fence_length: multiline_block_quote.fence_length,
                    fence_offset: multiline_block_quote.fence_offset,
                }),
                children,
            },

            NodeValue::Escaped => Self {
                data: ExNodeData::Escaped,
                children,
            },
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
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "front_matter".to_string(),
                    vec![ExNodeAttr(
                        "content".to_string(),
                        ExNodeAttrValue::Text(delimiter.to_string()),
                    )],
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
                data: ExNodeData::List(list),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "list".to_string(),
                    vec![
                        ExNodeAttr(
                            "list_type".to_string(),
                            ExNodeAttrValue::Text(list.list_type.to_string()),
                        ),
                        ExNodeAttr(
                            "marker_offset".to_string(),
                            ExNodeAttrValue::Usize(list.marker_offset),
                        ),
                        ExNodeAttr("padding".to_string(), ExNodeAttrValue::Usize(list.padding)),
                        ExNodeAttr("start".to_string(), ExNodeAttrValue::Usize(list.start)),
                        ExNodeAttr(
                            "delimiter".to_string(),
                            ExNodeAttrValue::Text(list.delimiter.to_string()),
                        ),
                        ExNodeAttr(
                            "bullet_char".to_string(),
                            ExNodeAttrValue::Text(char_to_string(list.bullet_char)),
                        ),
                        ExNodeAttr("tight".to_string(), ExNodeAttrValue::Bool(list.tight)),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // item
            ExNode {
                data: ExNodeData::Item(list),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "item".to_string(),
                    vec![
                        ExNodeAttr(
                            "list_type".to_string(),
                            ExNodeAttrValue::Text(list.list_type.to_string()),
                        ),
                        ExNodeAttr(
                            "marker_offset".to_string(),
                            ExNodeAttrValue::Usize(list.marker_offset),
                        ),
                        ExNodeAttr("padding".to_string(), ExNodeAttrValue::Usize(list.padding)),
                        ExNodeAttr("start".to_string(), ExNodeAttrValue::Usize(list.start)),
                        ExNodeAttr(
                            "delimiter".to_string(),
                            ExNodeAttrValue::Text(list.delimiter.to_string()),
                        ),
                        ExNodeAttr(
                            "bullet_char".to_string(),
                            ExNodeAttrValue::Text(char_to_string(list.bullet_char)),
                        ),
                        ExNodeAttr("tight".to_string(), ExNodeAttrValue::Bool(list.tight)),
                    ],
                    children.to_vec(),
                );
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
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "description_item".to_string(),
                    vec![
                        ExNodeAttr(
                            "marker_offset".to_string(),
                            ExNodeAttrValue::Usize(node_description_item.marker_offset),
                        ),
                        ExNodeAttr(
                            "padding".to_string(),
                            ExNodeAttrValue::Usize(node_description_item.padding),
                        ),
                    ],
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
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "code_block".to_string(),
                    vec![
                        ExNodeAttr(
                            "fenced".to_string(),
                            ExNodeAttrValue::Bool(code_block.fenced),
                        ),
                        ExNodeAttr(
                            "fence_char".to_string(),
                            ExNodeAttrValue::Text(char_to_string(code_block.fence_char)),
                        ),
                        ExNodeAttr(
                            "fence_length".to_string(),
                            ExNodeAttrValue::Usize(code_block.fence_length),
                        ),
                        ExNodeAttr(
                            "fence_offset".to_string(),
                            ExNodeAttrValue::Usize(code_block.fence_offset),
                        ),
                        ExNodeAttr(
                            "info".to_string(),
                            ExNodeAttrValue::Text(code_block.info.to_string()),
                        ),
                        ExNodeAttr(
                            "literal".to_string(),
                            ExNodeAttrValue::Text(code_block.literal.to_string()),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // html block
            ExNode {
                data: ExNodeData::HtmlBlock(html_block),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "html_block".to_string(),
                    vec![
                        ExNodeAttr(
                            "block_type".to_string(),
                            ExNodeAttrValue::U8(html_block.block_type),
                        ),
                        ExNodeAttr(
                            "literal".to_string(),
                            ExNodeAttrValue::Text(html_block.literal.to_string()),
                        ),
                    ],
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
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "heading".to_string(),
                    vec![
                        ExNodeAttr("level".to_string(), ExNodeAttrValue::U8(heading.level)),
                        ExNodeAttr("setext".to_string(), ExNodeAttrValue::Bool(heading.setext)),
                    ],
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
                data: ExNodeData::FootnoteDefinition(footnote_definition),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "footnote_definition".to_string(),
                    vec![
                        ExNodeAttr(
                            "name".to_string(),
                            ExNodeAttrValue::Text(footnote_definition.name.to_string()),
                        ),
                        ExNodeAttr(
                            "total_references".to_string(),
                            ExNodeAttrValue::U32(footnote_definition.total_references),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // table
            ExNode {
                data: ExNodeData::Table(table),
                children,
            } => {
                let alignments: Vec<String> = table
                    .alignments
                    .iter()
                    .map(|alignment| alignment.to_string())
                    .collect();

                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "table".to_string(),
                    vec![
                        ExNodeAttr("alignments".to_string(), ExNodeAttrValue::List(alignments)),
                        ExNodeAttr(
                            "num_columns".to_string(),
                            ExNodeAttrValue::Usize(table.num_columns),
                        ),
                        ExNodeAttr(
                            "num_rows".to_string(),
                            ExNodeAttrValue::Usize(table.num_rows),
                        ),
                        ExNodeAttr(
                            "num_nomempty_cells".to_string(),
                            ExNodeAttrValue::Usize(table.num_nonempty_cells),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // table row
            ExNode {
                data: ExNodeData::TableRow(header),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "table_row".to_string(),
                    vec![ExNodeAttr(
                        "header".to_string(),
                        ExNodeAttrValue::Text(header.to_string()),
                    )],
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

                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "task_item".to_string(),
                    vec![ExNodeAttr(
                        "symbol".to_string(),
                        ExNodeAttrValue::Text(symbol.to_string()),
                    )],
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
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "code".to_string(),
                    vec![
                        ExNodeAttr(
                            "num_backticks".to_string(),
                            ExNodeAttrValue::Usize(code.num_backticks),
                        ),
                        ExNodeAttr(
                            "literal".to_string(),
                            ExNodeAttrValue::Text(code.literal.to_string()),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // html inline
            ExNode {
                data: ExNodeData::HtmlInline(raw_html),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "code".to_string(),
                    vec![ExNodeAttr(
                        "raw_html".to_string(),
                        ExNodeAttrValue::Text(raw_html.to_string()),
                    )],
                    children.to_vec(),
                );
                doc.encode(env)
            }

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
                data: ExNodeData::Link(link),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "link".to_string(),
                    vec![
                        ExNodeAttr(
                            "url".to_string(),
                            ExNodeAttrValue::Text(link.url.to_string()),
                        ),
                        ExNodeAttr(
                            "title".to_string(),
                            ExNodeAttrValue::Text(link.title.to_string()),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // image
            ExNode {
                data: ExNodeData::Image(link),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "image".to_string(),
                    vec![
                        ExNodeAttr(
                            "url".to_string(),
                            ExNodeAttrValue::Text(link.url.to_string()),
                        ),
                        ExNodeAttr(
                            "title".to_string(),
                            ExNodeAttrValue::Text(link.title.to_string()),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // footnote reference
            ExNode {
                data: ExNodeData::FootnoteReference(footnote_reference),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "footnote_reference".to_string(),
                    vec![
                        ExNodeAttr(
                            "name".to_string(),
                            ExNodeAttrValue::Text(footnote_reference.name.to_string()),
                        ),
                        ExNodeAttr(
                            "ref_num".to_string(),
                            ExNodeAttrValue::U32(footnote_reference.ref_num),
                        ),
                        ExNodeAttr(
                            "ix".to_string(),
                            ExNodeAttrValue::U32(footnote_reference.ix),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }
            ExNode {
                data: ExNodeData::ShortCode(short_code),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "short_code".to_string(),
                    vec![
                        ExNodeAttr(
                            "name".to_string(),
                            ExNodeAttrValue::Text(short_code.shortcode.to_string()),
                        ),
                        ExNodeAttr(
                            "emoji".to_string(),
                            ExNodeAttrValue::Text(short_code.emoji.to_string()),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // short code

            // math
            ExNode {
                data: ExNodeData::Math(math),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "math".to_string(),
                    vec![
                        ExNodeAttr(
                            "dollar_math".to_string(),
                            ExNodeAttrValue::Bool(math.dollar_math),
                        ),
                        ExNodeAttr(
                            "display_math".to_string(),
                            ExNodeAttrValue::Bool(math.display_math),
                        ),
                        ExNodeAttr(
                            "literal".to_string(),
                            ExNodeAttrValue::Text(math.literal.to_string()),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // multiline block quote
            ExNode {
                data: ExNodeData::MultilineBlockQuote(multline_block_quote),
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) = (
                    "multiline_block_quote".to_string(),
                    vec![
                        ExNodeAttr(
                            "fence_length".to_string(),
                            ExNodeAttrValue::Usize(multline_block_quote.fence_length),
                        ),
                        ExNodeAttr(
                            "fence_offset".to_string(),
                            ExNodeAttrValue::Usize(multline_block_quote.fence_offset),
                        ),
                    ],
                    children.to_vec(),
                );
                doc.encode(env)
            }

            // escaped
            ExNode {
                data: ExNodeData::Escaped,
                children,
            } => {
                let doc: (String, ExNodeAttrs, ExNodeChildren) =
                    ("escaped".to_string(), vec![], children.to_vec());
                doc.encode(env)
            }
        }
    }
}

impl ExListType {
    fn from(list_type: ListType) -> Self {
        match list_type {
            ListType::Bullet => Self::Bullet,
            ListType::Ordered => Self::Ordered,
        }
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

impl ExListDelimType {
    fn from(list_delim_type: ListDelimType) -> Self {
        match list_delim_type {
            ListDelimType::Period => Self::Period,
            ListDelimType::Paren => Self::Paren,
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

fn char_to_string(c: u8) -> String {
    match String::from_utf8(vec![c]) {
        Ok(s) => s,
        Err(_) => "".to_string(),
    }
}

impl ToString for ExTableAlignment {
    fn to_string(&self) -> String {
        match self {
            Self::None => "none".to_string(),
            Self::Left => "left".to_string(),
            Self::Center => "center".to_string(),
            Self::Right => "right".to_string(),
        }
    }
}
