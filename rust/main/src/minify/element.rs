use std::collections::HashMap;

use crate::ast::{ElementClosingTag, NodeData};
use crate::cfg::Cfg;
use crate::minify::attr::{minify_attr, AttrMinified};
use crate::minify::content::minify_content;
use minify_html_common::spec::tag::ns::Namespace;
use minify_html_common::spec::tag::omission::{can_omit_as_before, can_omit_as_last_node};

#[derive(Copy, Clone, Eq, PartialEq)]
enum LastAttr {
    NoValue,
    Quoted,
    Unquoted,
}

pub fn minify_element(
    cfg: &Cfg,
    out: &mut Vec<u8>,
    descendant_of_pre: bool,
    ns: Namespace,
    // Use an empty slice if none.
    parent: &[u8],
    // Use an empty slice if the next element or text sibling node is not an element.
    next_sibling_as_element_tag_name: &[u8],
    // If the last node of the parent is an element and it's this one.
    is_last_child_text_or_element_node: bool,
    tag_name: &[u8],
    attributes: HashMap<Vec<u8>, Vec<u8>>,
    closing_tag: ElementClosingTag,
    children: Vec<NodeData>,
) {
    let can_omit_opening_tag = (tag_name == b"html" || tag_name == b"head")
        && attributes.is_empty()
        && !cfg.keep_html_and_head_opening_tags;
    let can_omit_closing_tag = !cfg.keep_closing_tags
        && (can_omit_as_before(tag_name, next_sibling_as_element_tag_name)
            || (is_last_child_text_or_element_node && can_omit_as_last_node(parent, tag_name)));

    // TODO Attributes list could become empty after minification, making opening tag eligible for omission again.
    if !can_omit_opening_tag {
        out.push(b'<');
        out.extend_from_slice(tag_name);
        let mut last_attr = LastAttr::NoValue;
        // TODO Further optimisation: order attrs based on optimal spacing strategy, given that spaces can be omitted after quoted attrs, and maybe after the tag name?
        let mut attrs_sorted = attributes.into_iter().collect::<Vec<_>>();
        attrs_sorted.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        for (name, value) in attrs_sorted {
            let min = minify_attr(cfg, ns, tag_name, &name, value);
            if let AttrMinified::Redundant = min {
                continue;
            };
            if cfg.keep_spaces_between_attributes || last_attr != LastAttr::Quoted {
                out.push(b' ');
            };
            out.extend_from_slice(&name);
            match min {
                AttrMinified::NoValue => {
                    last_attr = LastAttr::NoValue;
                }
                AttrMinified::Value(v) => {
                    debug_assert!(v.len() > 0);
                    out.push(b'=');
                    v.out(out);
                    last_attr = if v.quoted() {
                        LastAttr::Quoted
                    } else {
                        LastAttr::Unquoted
                    };
                }
                _ => unreachable!(),
            };
        }
        if closing_tag == ElementClosingTag::SelfClosing {
            if last_attr == LastAttr::Unquoted {
                out.push(b' ');
            };
            out.push(b'/');
        };
        out.push(b'>');
    }

    if closing_tag == ElementClosingTag::SelfClosing || closing_tag == ElementClosingTag::Void {
        debug_assert!(children.is_empty());
        return;
    };

    minify_content(
        cfg,
        out,
        descendant_of_pre || (ns == Namespace::Html && tag_name == b"pre"),
        tag_name,
        children,
    );

    if closing_tag != ElementClosingTag::Present || can_omit_closing_tag {
        return;
    };
    out.extend_from_slice(b"</");
    out.extend_from_slice(tag_name);
    out.push(b'>');
}
