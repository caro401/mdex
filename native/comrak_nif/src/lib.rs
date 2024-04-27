#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate rustler;

mod inkjet_adapter;
mod types;

use ammonia::clean;
use comrak::{markdown_to_html, markdown_to_html_with_plugins, ComrakPlugins, Options};
use inkjet_adapter::InkjetAdapter;
use rustler::{Env, NifResult, Term};
use types::nodes::*;
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

fn render(env: Env, unsafe_html: String, sanitize: bool) -> NifResult<Term> {
    let html = match sanitize {
        true => clean(&unsafe_html),
        false => unsafe_html,
    };

    rustler::serde::to_term(env, html).map_err(|err| err.into())
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
