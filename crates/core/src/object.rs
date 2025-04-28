use crate::gen::Layout;

pub type ObjectId = usize;

#[derive(Debug, Clone)]
pub struct Object {
    pub id: ObjectId,
    pub name: &'static str,
    pub capacity: usize,
    pub pickable: bool,
    pub x: usize,
    pub y: usize,
    pub contents: Vec<ObjectId>,
}

#[derive(Debug, Clone)]
pub struct ObjectSchema {
    pub capacity: usize,
    pub name: &'static str,
    pub pickable: bool,
    pub constraint: ObjectConstraint,
}

impl Default for ObjectSchema {
    fn default() -> Self {
        ObjectSchema {
            capacity: 0,
            name: "",
            pickable: false,
            constraint: ObjectConstraint::InRoom,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ObjectConstraint {
    InRoom,
    AdjacentObstacle,
    CloseToObstacle,
    And(Vec<ObjectConstraint>),
    Or(Vec<ObjectConstraint>),
    InsideOf(&'static [&'static str]),
}

impl ObjectConstraint {
    pub fn check(&self, layout: &Layout, x: usize, y: usize) -> bool {
        match self {
            ObjectConstraint::InRoom => layout.cells[y * layout.width + x] >= 0,
            ObjectConstraint::AdjacentObstacle => {
                if !ObjectConstraint::InRoom.check(layout, x, y) {
                    return false;
                }
                let w = layout.width;
                let h = layout.height;
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
            }
            ObjectConstraint::CloseToObstacle => {
                if !ObjectConstraint::InRoom.check(layout, x, y) {
                    return false;
                }
                let w = layout.width;
                let h = layout.height;
                for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
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
            }
            ObjectConstraint::And(constraints) => constraints.iter().all(|c| c.check(layout, x, y)),
            ObjectConstraint::Or(constraints) => constraints.iter().any(|c| c.check(layout, x, y)),
            ObjectConstraint::InsideOf(_) => false,  // placement only inside parent, not on floor
        }
    }
}

impl ObjectSchema {
    pub fn default_schemas() -> Vec<Self> {
        vec![
            // Storage Furniture
            ObjectSchema { name: "Wardrobe",       capacity: 10, pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Cupboard",       capacity: 10, pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Dresser",        capacity: 8,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Bookshelf",      capacity: 15, pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "KitchenCabinet", capacity: 12, pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Refrigerator",   capacity: 30, pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Dishwasher",     capacity: 15, pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Microwave",      capacity: 5,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Oven",           capacity: 10, pickable: false, constraint: ObjectConstraint::AdjacentObstacle },

            // Seating & Surfaces
            ObjectSchema { name: "Couch",          capacity: 10, pickable: false, constraint: ObjectConstraint::CloseToObstacle },
            ObjectSchema { name: "Sofa",           capacity: 0,  pickable: false, constraint: ObjectConstraint::CloseToObstacle },
            ObjectSchema { name: "Armchair",       capacity: 0,  pickable: false, constraint: ObjectConstraint::CloseToObstacle },
            ObjectSchema { name: "Chair",          capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "DiningTable",    capacity: 5,  pickable: false, constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "CoffeeTable",    capacity: 3,  pickable: false, constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Bed",            capacity: 0,  pickable: false, constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Nightstand",     capacity: 2,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },

            // Kitchen & Dining
            ObjectSchema { name: "Plate",          capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Mug",            capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Spoon",          capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Fork",           capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Knife",          capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Pan",            capacity: 0,  pickable: true,  constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Pot",            capacity: 0,  pickable: true,  constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Blender",        capacity: 0,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "CoffeeMaker",    capacity: 0,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },

            // Electronics & Decor
            ObjectSchema { name: "Television",     capacity: 0,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Remote",         capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Laptop",         capacity: 0,  pickable: true,  constraint: ObjectConstraint::InsideOf(&["Couch"]) },
            ObjectSchema { name: "Phone",          capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Clock",          capacity: 0,  pickable: false, constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "PictureFrame",   capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Mirror",         capacity: 0,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Speaker",        capacity: 0,  pickable: false, constraint: ObjectConstraint::InRoom },

            // Soft Furnishings & Plants
            ObjectSchema { name: "Rug",            capacity: 0,  pickable: false, constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Curtains",       capacity: 0,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Blanket",        capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Pillow",         capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Plant",          capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Vase",           capacity: 1,  pickable: true,  constraint: ObjectConstraint::InRoom },

            // Bathroom Fixtures
            ObjectSchema { name: "Toilet",         capacity: 0,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Sink",           capacity: 0,  pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Shower",         capacity: 0,  pickable: false, constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Bathtub",        capacity: 0,  pickable: false, constraint: ObjectConstraint::InRoom },

            // Miscellaneous
            ObjectSchema { name: "TrashCan",       capacity: 20, pickable: false, constraint: ObjectConstraint::AdjacentObstacle },
            ObjectSchema { name: "Book",           capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Toy",            capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Banana",         capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Apple",          capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
            ObjectSchema { name: "Bottle",         capacity: 0,  pickable: true,  constraint: ObjectConstraint::InRoom },
        ]
    }
}
