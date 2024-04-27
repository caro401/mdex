use crate::types::nodes::*;
use rustler::{Encoder, Env, Term};

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
