use crate::object::Object;
use crate::gen::World;

/// Describe an object and its contents in natural language
pub fn describe_object(obj: &Object, world: &World) -> String {
    let base = format!("The {} at ({}, {})", obj.name, obj.x, obj.y);
    if obj.contents.is_empty() {
        format!("{} contains nothing.", base)
    } else {
        let items: Vec<String> = obj
            .contents
            .iter()
            .filter_map(|id| world.objects.iter().find(|o| o.id == *id))
            .map(|o| format!("a {}", o.name))
            .collect();
        let list = match items.len() {
            1 => items[0].clone(),
            _ => {
                let last = items.last().unwrap().clone();
                let mut rest = items[..items.len()-1].join(", ");
                rest.push_str(" and ");
                rest.push_str(&last);
                rest
            }
        };
        format!("{} contains {}.", base, list)
    }
}