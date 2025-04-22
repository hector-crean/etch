use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;

use uuid::Uuid;


pub enum InjectUuidPolicy {
    Overwrite,
    KeepExisting,
}
pub struct InjectUuidVisitor { 
    policy: InjectUuidPolicy,
}

impl InjectUuidVisitor {
    pub fn new(policy: InjectUuidPolicy) -> Self {
        Self { policy }
    }
}

impl VisitMut for InjectUuidVisitor {
    fn visit_mut_jsx_element(&mut self, node: &mut JSXElement) {
        // Visit children first
        node.visit_mut_children_with(self);
        
        // Check if there's already a data-uuid attribute
        let has_uuid = node.opening.attrs.iter().any(|attr| {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(ident) = &jsx_attr.name {
                    return ident.sym == "data-uuid";
                }
            }
            false
        });
        
        // If we have a UUID and policy is KeepExisting, don't add a new one
        if has_uuid && matches!(self.policy, InjectUuidPolicy::KeepExisting) {
            return;
        }
        
        // Generate a UUID for this element
        let uuid = Uuid::new_v4().to_string();
        
        // Create a new data-uuid attribute
        let uuid_attr = JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName::from("data-uuid")),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: uuid.into(),
                raw: None,
            }))),
        });
        
        // If policy is Overwrite and there's an existing UUID, remove it first
        if matches!(self.policy, InjectUuidPolicy::Overwrite) {
            node.opening.attrs.retain(|attr| {
                if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                    if let JSXAttrName::Ident(ident) = &jsx_attr.name {
                        return ident.sym != "data-uuid";
                    }
                }
                true
            });
        }
        
        // Add the attribute to the opening element
        node.opening.attrs.push(uuid_attr);
    }
}






