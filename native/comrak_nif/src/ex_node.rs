use crate::types::nodes::*;
use crate::types::options::*;
use comrak::{
    nodes::{Ast, AstNode, LineColumn, NodeHeading, NodeList, NodeValue},
    Arena, Options,
};
use rustler::{types::tuple::get_tuple, Binary, Decoder, NifResult, Term};
use std::cell::RefCell;

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
