use crate::gen::Layout;

pub type ObjectId = usize;

#[derive(Debug, Clone)]
pub enum ObjectType {
    Wardrobe { capacity: usize },
    Cupboard { capacity: usize },
    Banana,
    Couch,
    // ... add more types ...
}

#[derive(Debug, Clone)]
pub struct ObjectSchema {
    pub typ: ObjectType,
    pub pickable: bool,
    pub constraint: fn(&Layout, usize, usize) -> bool,
}

impl ObjectSchema {
    pub fn default_schemas() -> Vec<Self> {
        vec![
            // one wardrobe (non-pickable) along walls
            ObjectSchema {
                typ: ObjectType::Wardrobe { capacity: 5 },
                pickable: false,
                constraint: |layout, x, y| {
                    let w = layout.width;
                    let h = layout.height;
                    let idx = y * w + x;
                    // must be in a room
                    if layout.cells[idx] < 0 { return false; }
                    // at least one 4-neighbor is an obstacle
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        if nx >= 0 && ny >= 0 && nx < w as isize && ny < h as isize {
                            let nidx = ny as usize * w + nx as usize;
                            if layout.cells[nidx] < 0 {
                                return true;
                            }
                        }
                    }
                    false
                },
            },
            // one cupboard in any room cell
            ObjectSchema {
                typ: ObjectType::Cupboard { capacity: 3 },
                pickable: false,
                constraint: |layout, x, y| layout.cells[y * layout.width + x] >= 0,
            },
            // two couches in rooms
            ObjectSchema {
                typ: ObjectType::Couch,
                pickable: false,
                constraint: |layout, x, y| layout.cells[y * layout.width + x] >= 0,
            },
            ObjectSchema {
                typ: ObjectType::Couch,
                pickable: false,
                constraint: |layout, x, y| layout.cells[y * layout.width + x] >= 0,
            },
            // three bananas in rooms
            ObjectSchema {
                typ: ObjectType::Banana,
                pickable: true,
                constraint: |layout, x, y| layout.cells[y * layout.width + x] >= 0,
            },
            ObjectSchema {
                typ: ObjectType::Banana,
                pickable: true,
                constraint: |layout, x, y| layout.cells[y * layout.width + x] >= 0,
            },
            ObjectSchema {
                typ: ObjectType::Banana,
                pickable: true,
                constraint: |layout, x, y| layout.cells[y * layout.width + x] >= 0,
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub id: ObjectId,
    pub typ: ObjectType,
    pub x: usize,
    pub y: usize,
    pub contents: Vec<ObjectId>,
    pub pickable: bool,
}
