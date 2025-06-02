use crate::gen::World;

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
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct ObjectSchema {
    pub capacity: usize,
    pub name: &'static str,
    pub pickable: bool,
    pub constraint: ObjectConstraint,
    pub description: &'static str,
    pub target: ObjectConstraint,
}

impl Default for ObjectSchema {
    fn default() -> Self {
        ObjectSchema {
            capacity: 0,
            name: "",
            pickable: false,
            constraint: ObjectConstraint::InRoom,
            description: "",
            target: ObjectConstraint::InRoom,
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
    WorldHas(&'static [&'static str]),
    InRoomNamed(&'static [&'static str]),
}

impl ObjectConstraint {
    /// Check constraint against current layout and placed objects
    pub fn check(&self, world: &World, x: usize, y: usize) -> bool {
        match self {
            ObjectConstraint::InRoom => world.layout.cells[y * world.layout.width + x] >= 0,
            ObjectConstraint::AdjacentObstacle => {
                if !ObjectConstraint::InRoom.check(world, x, y) {
                    return false;
                }
                let w = world.layout.width;
                let h = world.layout.height;
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && ny >= 0 && nx < w as isize && ny < h as isize {
                        let nidx = ny as usize * w + nx as usize;
                        if world.layout.cells[nidx] < 0 {
                            return true;
                        }
                    }
                }
                false
            }
            ObjectConstraint::CloseToObstacle => {
                if !ObjectConstraint::InRoom.check(world, x, y) {
                    return false;
                }
                let w = world.layout.width;
                let h = world.layout.height;
                for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && ny >= 0 && nx < w as isize && ny < h as isize {
                        let nidx = ny as usize * w + nx as usize;
                        if world.layout.cells[nidx] < 0 {
                            return true;
                        }
                    }
                }
                false
            }
            ObjectConstraint::And(constraints) => constraints.iter().all(|c| c.check(world, x, y)),
            ObjectConstraint::Or(constraints) => constraints.iter().any(|c| c.check(world, x, y)),
            ObjectConstraint::InsideOf(names) => {
                // allow placement only at parent coordinates with free capacity
                world.objects.iter().any(|o| {
                    names.contains(&o.name) && o.x == x && o.y == y && o.contents.len() < o.capacity
                })
            }
            ObjectConstraint::InRoomNamed(names) => {
                let idx = world.layout.cells[y * world.layout.width + x] as usize;
                names.contains(&world.layout.room_names[idx])
            }
            ObjectConstraint::WorldHas(names) => {
                // require at least one of named objects already placed
                world.objects.iter().any(|o| names.contains(&o.name))
            }
        }
    }
}

impl ObjectSchema {
    pub fn default_schemas() -> Vec<Self> {
        vec![
            // Fixtures & Furniture
            ObjectSchema {
                name: "TrashCan",
                capacity: 20,
                pickable: false,
                constraint: ObjectConstraint::InRoom,
                description: "A trash can.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Cupboard",
                capacity: 20,
                pickable: false,
                constraint: ObjectConstraint::AdjacentObstacle,
                description: "A kitchen cupboard.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "KitchenCabinet",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::And(vec![
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A kitchen cabinet.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Dishwasher",
                capacity: 20,
                pickable: false,
                constraint: ObjectConstraint::And(vec![
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A built-in dishwasher.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Refrigerator",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::And(vec![
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A refrigerator.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "FruitBowl",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&[
                    "Kitchen",
                    "Dining Room",
                    "Living Room",
                ]),
                description: "A bowl for holding fruit.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Drawer",
                capacity: 15,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Bedroom", "Office", "Study"]),
                description: "A sliding drawer unit.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "StorageBox",
                capacity: 30,
                pickable: false,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InRoomNamed(&["Hallway", "Living Room"]),
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A box for loose items.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "DiningTable",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Dining Room", "Kitchen"]),
                description: "A dining table.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "CoffeeTable",
                capacity: 5,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Living Room"]),
                description: "A coffee table.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Bookshelf",
                capacity: 30,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&[
                    "Living Room",
                    "Study",
                    "Bedroom",
                    "Office",
                ]),
                description: "A bookshelf.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "TVStand",
                capacity: 5,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Living Room"]),
                description: "A TV stand.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Sofa",
                capacity: 3,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Living Room", "Guest Room"]),
                description: "A sofa.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Armchair",
                capacity: 1,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Living Room", "Study"]),
                description: "An armchair.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Bed",
                capacity: 5,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Bedroom", "Guest Room"]),
                description: "A bed.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Wardrobe",
                capacity: 20,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Bedroom", "Guest Room"]),
                description: "A wardrobe.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Dresser",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Bedroom", "Guest Room"]),
                description: "A dresser.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Desk",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Study", "Office", "Bedroom"]),
                description: "A desk.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Nightstand",
                capacity: 5,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Bedroom", "Guest Room"]),
                description: "A nightstand.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "ToyBox",
                capacity: 50,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Playroom", "Living Room"]),
                description: "A toy box.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "BathroomCabinet",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Bathroom"]),
                description: "A bathroom cabinet.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "KeyHolder",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Hallway", "Office"]),
                description: "A wall-mounted key holder.",
                target: ObjectConstraint::InRoom,
            },
            // Kitchen Utensils
            ObjectSchema {
                name: "Spatula",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&[
                        "Drawer",
                        "Cupboard",
                        "Dishwasher",
                        "DiningTable",
                    ]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A spatula for flipping food.",
                target: ObjectConstraint::InsideOf(&["Drawer", "Cupboard", "Dishwasher"]),
            },
            ObjectSchema {
                name: "Whisk",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer", "Cupboard", "Dishwasher"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A whisk for mixing ingredients.",
                target: ObjectConstraint::InsideOf(&["Drawer", "Cupboard", "Dishwasher"]),
            },
            ObjectSchema {
                name: "CookingPot",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "Dishwasher", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A large cooking pot.",
                target: ObjectConstraint::InsideOf(&["Cupboard", "Dishwasher"]),
            },
            ObjectSchema {
                name: "FryingPan",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "Dishwasher", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A frying pan.",
                target: ObjectConstraint::InsideOf(&["Cupboard", "Dishwasher"]),
            },
            ObjectSchema {
                name: "CuttingBoard",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer", "Cupboard", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A wooden cutting board.",
                target: ObjectConstraint::InsideOf(&["Drawer", "Cupboard"]),
            },
            ObjectSchema {
                name: "Dirty CuttingBoard",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A wooden cutting board.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "Kettle",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "An electric kettle.",
                target: ObjectConstraint::InsideOf(&["Cupboard"]),
            },
            ObjectSchema {
                name: "Blender",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A countertop blender.",
                target: ObjectConstraint::InsideOf(&["Cupboard"]),
            },
            ObjectSchema {
                name: "Toaster",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A two-slice toaster.",
                target: ObjectConstraint::InsideOf(&["Cupboard"]),
            },
            ObjectSchema {
                name: "Microwave",
                capacity: 1,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Kitchen"]),
                description: "A microwave oven.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "MixingBowl",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A ceramic mixing bowl.",
                target: ObjectConstraint::InsideOf(&["Cupboard"]),
            },
            // Food & Pantry
            ObjectSchema {
                name: "Apple",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["FruitBowl"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                ]),
                description: "A fresh red apple.",
                target: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["FruitBowl"]),
                    ObjectConstraint::InsideOf(&["Refrigerator"]),
                ]),
            },
            ObjectSchema {
                name: "Orange",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["FruitBowl"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A juicy orange.",
                target: ObjectConstraint::InsideOf(&["FruitBowl"]),
            },
            ObjectSchema {
                name: "MilkCarton",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Refrigerator"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A carton of milk.",
                target: ObjectConstraint::InsideOf(&["Refrigerator"]),
            },
            ObjectSchema {
                name: "Egg",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Refrigerator"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A chicken egg.",
                target: ObjectConstraint::InsideOf(&["Refrigerator"]),
            },
            ObjectSchema {
                name: "CerealBox",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A box of cereal.",
                target: ObjectConstraint::InsideOf(&["Cupboard"]),
            },
            ObjectSchema {
                name: "BreadLoaf",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A loaf of bread.",
                target: ObjectConstraint::InsideOf(&["Cupboard"]),
            },
            ObjectSchema {
                name: "CheeseBlock",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["Refrigerator"]),
                description: "A block of cheese.",
                target: ObjectConstraint::InsideOf(&["Refrigerator"]),
            },
            ObjectSchema {
                name: "YogurtCup",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["Refrigerator"]),
                description: "A cup of yogurt.",
                target: ObjectConstraint::InsideOf(&["Refrigerator"]),
            },
            ObjectSchema {
                name: "JuiceBottle",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["Refrigerator"]),
                description: "A bottle of juice.",
                target: ObjectConstraint::InsideOf(&["Refrigerator"]),
            },
            ObjectSchema {
                name: "WaterBottle",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Refrigerator"]),
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A bottle of water.",
                target: ObjectConstraint::InsideOf(&["Refrigerator"]),
            },
            // Bathroom Essentials
            ObjectSchema {
                name: "ShampooBottle",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["BathroomCabinet"]),
                    ObjectConstraint::InRoomNamed(&["Bathroom"]),
                ]),
                description: "A bottle of shampoo.",
                target: ObjectConstraint::InsideOf(&["BathroomCabinet"]),
            },
            ObjectSchema {
                name: "SoapBar",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["BathroomCabinet"]),
                    ObjectConstraint::InRoomNamed(&["Bathroom"]),
                ]),
                description: "A bar of soap.",
                target: ObjectConstraint::InsideOf(&["BathroomCabinet"]),
            },
            ObjectSchema {
                name: "Hairbrush",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer"]),
                    ObjectConstraint::InRoomNamed(&["Bathroom"]),
                ]),
                description: "A hairbrush.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "Razor",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["BathroomCabinet"]),
                    ObjectConstraint::InRoomNamed(&["Bathroom"]),
                ]),
                description: "A shaving razor.",
                target: ObjectConstraint::InsideOf(&["BathroomCabinet"]),
            },
            ObjectSchema {
                name: "Towel",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["BathroomCabinet"]),
                    ObjectConstraint::InRoomNamed(&["Bathroom"]),
                ]),
                description: "A bath towel.",
                target: ObjectConstraint::InsideOf(&["BathroomCabinet"]),
            },
            ObjectSchema {
                name: "Toothpaste",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["BathroomCabinet"]),
                    ObjectConstraint::InRoomNamed(&["Bathroom"]),
                ]),
                description: "A tube of toothpaste.",
                target: ObjectConstraint::InsideOf(&["BathroomCabinet"]),
            },
            ObjectSchema {
                name: "ToothbrushHolder",
                capacity: 5,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Bathroom"]),
                description: "A stand for toothbrushes.",
                target: ObjectConstraint::InRoomNamed(&["Bathroom"]),
            },
            ObjectSchema {
                name: "Toothbrush",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["ToothbrushHolder", "BathroomCabinet"]),
                    ObjectConstraint::InRoomNamed(&["Bathroom"]),
                ]),
                description: "A toothbrush.",
                target: ObjectConstraint::InsideOf(&["ToothbrushHolder"]),
            },
            ObjectSchema {
                name: "BathMat",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InRoomNamed(&["Bathroom"]),
                description: "A mat outside the tub.",
                target: ObjectConstraint::InRoomNamed(&["Bathroom"]),
            },
            // Office Supplies
            ObjectSchema {
                name: "Stapler",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer"]),
                    ObjectConstraint::InRoomNamed(&["Office", "Study"]),
                ]),
                description: "A paper stapler.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "PaperStack",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Desk"]),
                    ObjectConstraint::InRoomNamed(&["Office", "Study"]),
                ]),
                description: "A stack of loose papers.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "Envelope",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Desk"]),
                    ObjectConstraint::InRoomNamed(&["Office", "Study"]),
                ]),
                description: "A paper envelope.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "Calculator",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Desk"]),
                    ObjectConstraint::InRoomNamed(&["Office", "Study"]),
                ]),
                description: "A desk calculator.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "Mouse",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Desk"]),
                    ObjectConstraint::InRoomNamed(&["Office", "Study"]),
                ]),
                description: "A computer mouse.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "Keyboard",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Desk"]),
                    ObjectConstraint::InRoomNamed(&["Office", "Study"]),
                ]),
                description: "A computer keyboard.",
                target: ObjectConstraint::InsideOf(&["Desk"]),
            },
            ObjectSchema {
                name: "Monitor",
                capacity: 0,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Office", "Study"]),
                description: "A computer monitor.",
                target: ObjectConstraint::InRoomNamed(&["Office", "Study"]),
            },
            // Cleaning Supplies
            ObjectSchema {
                name: "Broom",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A broom for sweeping.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "Mop",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A mop for mopping floors.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "VacuumCleaner",
                capacity: 0,
                pickable: false,
                constraint: ObjectConstraint::AdjacentObstacle,
                description: "An upright vacuum cleaner.",
                target: ObjectConstraint::AdjacentObstacle,
            },
            ObjectSchema {
                name: "Bucket",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A cleaning bucket.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "SprayBottle",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A bottle of cleaning spray.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "Sponge",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A cleaning sponge.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            // Personal Items
            ObjectSchema {
                name: "Wallet",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer", "StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A personal wallet.",
                target: ObjectConstraint::InsideOf(&["Drawer", "StorageBox"]),
            },
            ObjectSchema {
                name: "Sunglasses",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer"]),
                    ObjectConstraint::InRoomNamed(&["Hallway", "Living Room"]),
                ]),
                description: "A pair of sunglasses.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "Watch",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A wristwatch.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "Backpack",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A shoulder backpack.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "Umbrella",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A closed umbrella.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            // Children’s Toys
            ObjectSchema {
                name: "PuzzlePiece",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["ToyBox"]),
                    ObjectConstraint::InRoomNamed(&["Playroom"]),
                ]),
                description: "A piece of a jigsaw puzzle.",
                target: ObjectConstraint::InsideOf(&["ToyBox"]),
            },
            ObjectSchema {
                name: "LegoBrick",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["ToyBox"]),
                    ObjectConstraint::InRoomNamed(&["Playroom"]),
                ]),
                description: "A single lego brick.",
                target: ObjectConstraint::InsideOf(&["ToyBox"]),
            },
            ObjectSchema {
                name: "Ball",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["ToyBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A small ball.",
                target: ObjectConstraint::InsideOf(&["ToyBox"]),
            },
            ObjectSchema {
                name: "Doll",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["ToyBox", "Sofa", "Armchair"]),
                    ObjectConstraint::InRoomNamed(&["Playroom"]),
                ]),
                description: "A child’s doll.",
                target: ObjectConstraint::InsideOf(&["ToyBox"]),
            },
            ObjectSchema {
                name: "BoardGame",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InsideOf(&["DiningTable", "CoffeeTable"]),
                ]),
                description: "A board game set.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "Crayon",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoomNamed(&["Playroom"]),
                ]),
                description: "A colored crayon.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "PaintBrush",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoomNamed(&["Playroom"]),
                ]),
                description: "A paint brush.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            // Decor & Electronics
            ObjectSchema {
                name: "Vase",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["CoffeeTable", "DiningTable"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A decorative vase.",
                target: ObjectConstraint::InsideOf(&["CoffeeTable", "DiningTable"]),
            },
            ObjectSchema {
                name: "PictureFrame",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["CoffeeTable", "DiningTable"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A photo in a frame.",
                target: ObjectConstraint::InsideOf(&["CoffeeTable", "DiningTable"]),
            },
            ObjectSchema {
                name: "Lamp",
                capacity: 0,
                pickable: false,
                constraint: ObjectConstraint::AdjacentObstacle,
                description: "A floor lamp.",
                target: ObjectConstraint::AdjacentObstacle,
            },
            ObjectSchema {
                name: "Rug",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InRoom,
                description: "A small rug.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Cushion",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Sofa", "Bed"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A throw cushion.",
                target: ObjectConstraint::InsideOf(&["Sofa", "Bed"]),
            },
            ObjectSchema {
                name: "GameController",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["StorageBox"]),
                    ObjectConstraint::InRoomNamed(&["Living Room", "Office"]),
                ]),
                description: "A video game controller.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "Headphones",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer", "StorageBox"]),
                    ObjectConstraint::InRoomNamed(&["Office", "Living Room"]),
                ]),
                description: "A pair of headphones.",
                target: ObjectConstraint::InsideOf(&["Drawer", "StorageBox"]),
            },
            ObjectSchema {
                name: "Speaker",
                capacity: 0,
                pickable: false,
                constraint: ObjectConstraint::AdjacentObstacle,
                description: "A Bluetooth speaker.",
                target: ObjectConstraint::AdjacentObstacle,
            },
            ObjectSchema {
                name: "ChargingCable",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Drawer", "StorageBox"]),
                    ObjectConstraint::InRoom,
                ]),
                description: "A USB charging cable.",
                target: ObjectConstraint::InsideOf(&["Drawer", "StorageBox"]),
            },
            // ——— Dirty kitchen items for the Dishwasher ———
            ObjectSchema {
                name: "DirtyPlate",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["DiningTable", "Cupboard"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                ]),
                description: "A ceramic plate with leftover food scraps.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "DirtyBowl",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["DiningTable", "Cupboard"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                ]),
                description: "A bowl stained with sauce or soup.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "DirtyCup",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["DiningTable", "Cupboard"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                ]),
                description: "A cup with tea or coffee stains.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "DirtySilverware",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                ]),
                description: "A spoon, fork, or knife covered in food residue.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            // ——— More Dirty kitchen items for the Dishwasher ———
            ObjectSchema {
                name: "DirtyGlass",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                ]),
                description: "A drinking glass with lipstick or juice stains.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "DirtyWineGlass",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                ]),
                description: "A wine glass with dried wine residue.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "DirtyMug",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Breakfast Nook"]),
                ]),
                description: "A coffee mug with grounds and stains.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "DirtySaucepan",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "StoveTop"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A saucepan caked with burnt-on sauce.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "DirtyBakingTray",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "Oven"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A baking tray with hardened batter or crumbs.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            ObjectSchema {
                name: "DirtyColander",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "Sink Area"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A colander with stuck-on vegetable bits.",
                target: ObjectConstraint::InsideOf(&["Dishwasher"]),
            },
            // ——— Spoiled fruits & vegetables for the TrashCan ———
            ObjectSchema {
                name: "RottenTomato",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["FruitBowl", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                ]),
                description: "A tomato that has gone mushy and moldy.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "MoldyBread",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A loaf of bread covered in green or white mold.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "SpoiledLettuce",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Refrigerator", "FruitBowl"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A head of lettuce that’s wilted and slimy.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "RottenBanana",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["FruitBowl", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A banana blackened with overripeness and rot.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            // ——— Even more spoiled fruits & vegetables for the TrashCan ———
            ObjectSchema {
                name: "RottenStrawberry",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["FruitBowl", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A strawberry covered in mold and leaking juices.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "RottenGrapes",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["FruitBowl"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A bunch of grapes that have shriveled and rotten.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "SpoiledCucumber",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Refrigerator", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A cucumber that’s gone soft and slimy.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "SpoiledCarrot",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Refrigerator", "VegetableCrisper"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A carrot that’s limp and discolored.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "RottenPotato",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Pantry", "Cupboard"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A potato speckled with soft spots and mold.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "RottenOnion",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Pantry", "Cupboard"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "An onion with a slimy, foul-smelling rot core.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "MoldyBreadSlice",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["BreadLoaf", "DiningTable"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "A single slice of bread covered in fuzzy mold.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            ObjectSchema {
                name: "RottenBlueberries",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["FruitBowl"]),
                    ObjectConstraint::InRoomNamed(&["Kitchen"]),
                ]),
                description: "Blueberries that have burst and fermented.",
                target: ObjectConstraint::InsideOf(&["TrashCan"]),
            },
            // ——— Laundry & Clothing ———
            ObjectSchema {
                name: "LaundryBasket",
                capacity: 50,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Laundry Room", "Bathroom"]),
                description: "A basket for holding dirty laundry.",
                target: ObjectConstraint::InRoomNamed(&["Laundry Room", "Bathroom"]),
            },
            ObjectSchema {
                name: "DirtyClothes",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&[
                    "LaundryBasket",
                    "Wardrobe",
                    "StorageBox",
                ]),
                description: "A pile of worn garments needing washing.",
                target: ObjectConstraint::InsideOf(&["LaundryBasket"]),
            },
            ObjectSchema {
                name: "IroningBoard",
                capacity: 10,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Laundry Room", "Bedroom"]),
                description: "A fold-out ironing board.",
                target: ObjectConstraint::InRoomNamed(&["Laundry Room", "Bedroom"]),
            },
            ObjectSchema {
                name: "Iron",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Cupboard", "Drawer"]),
                    ObjectConstraint::InRoomNamed(&["Laundry Room", "Bedroom"]),
                ]),
                description: "An electric clothes iron.",
                target: ObjectConstraint::InsideOf(&["Cupboard", "Drawer"]),
            },
            // ——— Safety & First Aid ———
            ObjectSchema {
                name: "FireExtinguisher",
                capacity: 5,
                pickable: false,
                constraint: ObjectConstraint::AdjacentObstacle,
                description: "A wall-mounted fire extinguisher.",
                target: ObjectConstraint::AdjacentObstacle,
            },
            ObjectSchema {
                name: "FirstAidKit",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InRoomNamed(&["Bathroom", "Hallway", "Office"]),
                description: "A box of first-aid supplies.",
                target: ObjectConstraint::InRoomNamed(&["Bathroom", "Hallway", "Office"]),
            },
            // ——— Pet Supplies ———
            ObjectSchema {
                name: "PetFoodBowl",
                capacity: 10,
                pickable: true,
                constraint: ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
                description: "A bowl for pet food or water.",
                target: ObjectConstraint::InRoomNamed(&["Kitchen", "Dining Room"]),
            },
            ObjectSchema {
                name: "PetBed",
                capacity: 5,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Living Room", "Bedroom"]),
                description: "A cushioned pet bed.",
                target: ObjectConstraint::InRoomNamed(&["Living Room", "Bedroom"]),
            },
            ObjectSchema {
                name: "DogLeash",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["Drawer", "StorageBox"]),
                description: "A leash for walking a dog.",
                target: ObjectConstraint::InsideOf(&["Drawer", "StorageBox"]),
            },
            // ——— Tools & Maintenance ———
            ObjectSchema {
                name: "Hammer",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["GarageShelf", "StorageBox"]),
                description: "A standard claw hammer.",
                target: ObjectConstraint::InsideOf(&["GarageShelf", "StorageBox"]),
            },
            ObjectSchema {
                name: "ScrewdriverSet",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["GarageShelf", "Drawer"]),
                description: "A set of screwdrivers in a pouch.",
                target: ObjectConstraint::InsideOf(&["GarageShelf", "Drawer"]),
            },
            ObjectSchema {
                name: "Toolbox",
                capacity: 30,
                pickable: false,
                constraint: ObjectConstraint::InRoomNamed(&["Garage", "Basement"]),
                description: "A portable metal toolbox.",
                target: ObjectConstraint::InRoomNamed(&["Garage", "Basement"]),
            },
            // ——— Decor Items (wall-adjacent) ———
            ObjectSchema {
                name: "WallMirror",
                capacity: 0,
                pickable: false,
                constraint: ObjectConstraint::And(vec![
                    ObjectConstraint::InRoomNamed(&["Living Room", "Bedroom", "Hallway"]),
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A decorative wall mirror with an ornate frame.",
                target: ObjectConstraint::InRoomNamed(&["Living Room", "Bedroom", "Hallway"]),
            },
            ObjectSchema {
                name: "WallClock",
                capacity: 0,
                pickable: false,
                constraint: ObjectConstraint::And(vec![
                    ObjectConstraint::InRoom,
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A round analog wall clock.",
                target: ObjectConstraint::InRoom,
            },
            ObjectSchema {
                name: "Chandelier",
                capacity: 0,
                pickable: false,
                constraint: ObjectConstraint::AdjacentObstacle,
                description: "An elegant ceiling chandelier.",
                target: ObjectConstraint::AdjacentObstacle,
            },
            ObjectSchema {
                name: "Curtains",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::And(vec![
                    ObjectConstraint::InRoomNamed(&["Living Room", "Bedroom"]),
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A pair of window curtains.",
                target: ObjectConstraint::InRoomNamed(&["Living Room", "Bedroom"]),
            },
            ObjectSchema {
                name: "Blinds",
                capacity: 0,
                pickable: false,
                constraint: ObjectConstraint::And(vec![
                    ObjectConstraint::InRoomNamed(&["Office", "Living Room"]),
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A set of horizontal window blinds.",
                target: ObjectConstraint::InRoomNamed(&["Office", "Living Room"]),
            },
            ObjectSchema {
                name: "Tapestry",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::And(vec![
                    ObjectConstraint::InRoomNamed(&["Living Room", "Dining Room"]),
                    ObjectConstraint::AdjacentObstacle,
                ]),
                description: "A decorative woven wall tapestry.",
                target: ObjectConstraint::InRoomNamed(&["Living Room", "Dining Room"]),
            },
            // ——— Revised Pickable Decor Items ———
            ObjectSchema {
                name: "ThrowBlanket",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::Or(vec![
                    ObjectConstraint::InsideOf(&["Sofa", "Armchair", "Bed"]),
                    ObjectConstraint::InRoomNamed(&["Living Room", "Bedroom"]),
                ]),
                description: "A cozy knit throw blanket.",
                target: ObjectConstraint::InsideOf(&["Wardrobe", "Dresser", "StorageBox"]),
            },
            ObjectSchema {
                name: "DecorativeBowl",
                capacity: 5,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["CoffeeTable", "DiningTable", "Shelf"]),
                description: "A ceramic bowl used purely for decoration.",
                target: ObjectConstraint::InsideOf(&["Cupboard", "Drawer", "StorageBox"]),
            },
            ObjectSchema {
                name: "CoasterSet",
                capacity: 4,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["CoffeeTable", "SideTable"]),
                description: "A set of drink coasters.",
                target: ObjectConstraint::InsideOf(&["Drawer"]),
            },
            ObjectSchema {
                name: "Sculpture",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["Shelf", "CoffeeTable", "Mantel"]),
                description: "A small decorative sculpture or figurine.",
                target: ObjectConstraint::InsideOf(&["StorageBox"]),
            },
            ObjectSchema {
                name: "FairyLights",
                capacity: 0,
                pickable: true,
                constraint: ObjectConstraint::InRoom,
                description: "A string of decorative fairy lights.",
                target: ObjectConstraint::InsideOf(&["StorageBox", "Drawer"]),
            },
            ObjectSchema {
                name: "PhotoAlbum",
                capacity: 20,
                pickable: true,
                constraint: ObjectConstraint::InsideOf(&["CoffeeTable", "Bookshelf", "Desk"]),
                description: "A leather-bound photo album.",
                target: ObjectConstraint::InsideOf(&["Bookshelf", "Drawer"]),
            },
        ]
    }
}

impl Object {
    /// return true if this object is in its correct target placement
    pub fn check_placement(&self, world: &World) -> bool {
        // find schema for this object name
        let schema = ObjectSchema::default_schemas()
            .into_iter()
            .find(|s| s.name == self.name)
            .expect("Schema must exist for object");

        // check target constraint
        if !schema.target.check(world, self.x, self.y) {
            return false;
        }
        true
    }
}
