use rustler::{NifTuple, NifUntaggedEnum};

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
