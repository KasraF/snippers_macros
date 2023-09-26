#![allow(dead_code, unused_variables, unused_imports)]

use std::rc::Rc;

use proc_macro::{TokenStream, TokenTree};
use to_snake_case::ToSnakeCase;

#[proc_macro_attribute]
pub fn derive_store(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut rs = item.to_string();

    for token in attr.into_iter() {
        match token {
            TokenTree::Ident(ident) => {
                let field = ident.to_string().to_snake_case();

                let impl_str = format!(
                    "
impl ProgramStore<{ident}> for Store {{
    fn insert(&mut self, node: Box<dyn MaybeNode<{ident}>>) -> Option<Index<{ident}>> {{
        // Check if it's unique
        let values = node.values();
        assert!(
            values.len() == self.ex,
        );
        if self.{field}_oe.contains(values) {{
            return None;
        }}

        // The values for this node are unique. So let's go!!!
        let nodes = &mut self.{field}s;
        let nodes_len = nodes.len();
        let idx = Index::new(nodes_len);
        let (node, mut node_values) = node.to_node(idx);

        // Add the node
        nodes.push(node);

        // Add the values
        // TODO This .clone() **hurts**. Can we do anything about it?!
        self.{field}_oe.insert(node_values.clone());
        let values = &mut self.{field}_vals;
        assert!(
            values.len() == *idx * self.ex,
        );
        values.append(&mut node_values);

        Some(idx)
    }}

    fn program<'s>(&'s self, idx: Index<{ident}>) -> &'s dyn Node<{ident}> {{
        self.{field}s[*idx].as_ref()
    }}

    fn values<'s>(&'s self, idx: Index<{ident}>) -> &'s [{ident}] {{
        self.{field}_vals[*idx * self.ex..(*idx + 1) * self.ex].as_ref()
    }}

    fn has(&self, idx: Index<{ident}>) -> bool {{
        self.{field}s.len() > *idx
    }}
}}
        
"
                );
                rs += &impl_str;
            }
            TokenTree::Punct(punct) if punct.as_char() == ',' => (),
            _ => {
                panic!("Unrecognized token: {token}")
            }
        }
    }

    rs.parse().unwrap()
}
