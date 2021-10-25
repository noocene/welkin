use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::edit::UiSectionVariance;

use super::{zipper::Cursor, UiSection};

pub fn render_to(data: &Cursor<UiSection>, node: &Node) -> Result<(), JsValue> {
    match &data {
        Cursor::Lambda(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Lambda(cursor.clone()))?
                .unwrap();

            render_to(&cursor.clone().body(), &node)?;
        }
        Cursor::Application(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Application(cursor.clone()))?
                .unwrap();

            let function_node = node.child_nodes().get(0).unwrap();
            let argument_node = node.child_nodes().get(1).unwrap();
            render_to(&cursor.clone().function(), &function_node)?;
            render_to(&cursor.clone().argument(), &argument_node)?;
        }
        Cursor::Put(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Put(cursor.clone()))?
                .unwrap();

            render_to(&cursor.clone().term(), &node)?;
        }
        Cursor::Reference(cursor) => {
            let annotation = cursor.annotation();
            annotation.render(node, &Cursor::Reference(cursor.clone()))?;
        }
        Cursor::Duplication(cursor) => {
            let annotation = cursor.annotation();

            let node = annotation
                .render(node, &Cursor::Duplication(cursor.clone()))?
                .unwrap();

            let expression_node = node.child_nodes().get(1).unwrap();
            let body_node = node.child_nodes().get(2).unwrap();
            render_to(&cursor.clone().expression(), &expression_node)?;
            render_to(&cursor.clone().body(), &body_node)?;
        }
        Cursor::Universe(cursor) => {
            let annotation = cursor.annotation();
            annotation.render(node, &Cursor::Universe(cursor.clone()))?;
        }
        Cursor::Function(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Function(cursor.clone()))?
                .unwrap();

            let argument_type_node = node.child_nodes().get(2).unwrap();
            let return_type_node = node.child_nodes().get(3).unwrap();
            render_to(&cursor.clone().argument_type(), &argument_type_node)?;
            render_to(&cursor.clone().return_type(), &return_type_node)?;
        }
        Cursor::Wrap(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Wrap(cursor.clone()))?
                .unwrap();

            render_to(&cursor.clone().term(), &node)?;
        }

        Cursor::Hole(cursor) => {
            let annotation = cursor.annotation();
            annotation.render(node, &Cursor::Hole(cursor.clone()))?;
        }

        Cursor::Dynamic(cursor) => {
            cursor.term.render_to(
                &cursor.up,
                match &cursor.annotation.variant {
                    UiSectionVariance::Dynamic(variance) => variance.as_ref(),
                    _ => panic!(),
                },
                node,
            )?;
        }
    }

    Ok(())
}
