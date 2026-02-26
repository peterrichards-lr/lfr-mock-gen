use edit_xml::{Document, Element};

pub fn find_elements_by_name(doc: &Document, name: &str) -> Vec<Element> {
    let mut results = Vec::new();
    let root = match doc.root_element() {
        Some(r) => r,
        None => return results,
    };

    let mut stack: Vec<Element> = vec![root];
    while let Some(el) = stack.pop() {
        if el.name(doc) == name {
            results.push(el);
        }
        for child in el.children(doc) {
            if let Some(child_el) = child.as_element() {
                stack.push(child_el);
            }
        }
    }
    results
}
