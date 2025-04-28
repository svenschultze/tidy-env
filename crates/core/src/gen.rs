use rand::{Rng, SeedableRng};
use rand::seq::IteratorRandom;
use rand::rngs::StdRng;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::object::{Object, ObjectSchema, ObjectConstraint};
use crate::{
    OUTSIDE, WALL, CLOSED_DOOR
};

pub type Cell = i8;

#[derive(Clone, Copy)]
pub struct GenOpts {
    pub seed: u64,
    pub max_rooms: usize,
    pub width: usize,    // number of columns
    pub height: usize,   // number of rows
    pub max_objects: usize, // maximum number of objects to place
}

#[derive(Debug)]
pub struct Layout {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl Layout {
    pub fn new(width: usize, height: usize, cells: Vec<Cell>) -> Self {
        Self { width, height, cells }
    }
}

const MIN_THICK_CELLS: usize = 3;
const MIN_ROOM_AREA_CELLS: usize = 24;

const MIN_SIZE_RATIO: f64 = 0.2;
const MIN_AR: f64 = 1.2;
const MAX_AR: f64 = 4.0;

const SAMPLE_C: usize = 10;
const DOOR_MIN_CELLS: usize = 2;
const DOOR_MAX_CELLS: usize = 4;



fn find_runs(arr: &[bool]) -> usize {
    let mut runs = 0;
    let mut prev = false;
    for &v in arr {
        if v && !prev {
            runs += 1;
        }
        prev = v;
    }
    runs
}

fn thinnest_segment_length(mask: &[bool], width: usize, height: usize) -> (usize, usize) {
    let mut min_w = width;
    let mut min_h = height;
    // rows
    for y in 0..height {
        let row = &mask[y * width..y * width + width];
        if find_runs(row) > 0 {
            let mut start = None;
            for x in 0..=width {
                if x < width && row[x] {
                    start.get_or_insert(x);
                } else if let Some(s) = start {
                    min_w = min_w.min(x - s);
                    start = None;
                }
            }
        }
    }
    // cols
    for x in 0..width {
        let mut start = None;
        for y in 0..=height {
            let v = if y < height { mask[y * width + x] } else { false };
            if v {
                start.get_or_insert(y);
            } else if let Some(s) = start {
                min_h = min_h.min(y - s);
                start = None;
            }
        }
    }
    (min_w, min_h)
}

/// Region of contiguous open cells
#[derive(Clone)]
struct Region {
    mask: Vec<bool>,
    area: usize,
    bbox: (usize, usize, usize, usize), // (miny, maxy, minx, maxx)
}

impl Region {
    /// Build region from mask with given width/height
    fn new(mask: Vec<bool>, width: usize, height: usize) -> Self {
        let mut ys = Vec::new();
        let mut xs = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if mask[y * width + x] {
                    ys.push(y);
                    xs.push(x);
                }
            }
        }
        let miny = *ys.iter().min().unwrap();
        let maxy = *ys.iter().max().unwrap();
        let minx = *xs.iter().min().unwrap();
        let maxx = *xs.iter().max().unwrap();
        let area = mask.iter().filter(|&&b| b).count();
        Self { mask, area, bbox: (miny, maxy, minx, maxx) }
    }
}

fn make_concave_shell(width: usize, height: usize, seed: u64) -> Vec<bool> {
    let mut rng = StdRng::seed_from_u64(seed);
    // initial full shell mask
    let mut shell = vec![true; width * height];
    // choose concave corner size between 3 and 5 cells
    let w1 = rng.gen_range(3..=5).min(width);
    let h1 = rng.gen_range(3..=5).min(height);
    let corner = rng.gen_range(0..4);
    for y in 0..h1 {
        for x in 0..w1 {
            let idx = match corner {
                0 => y * width + x,
                1 => y * width + (width - w1 + x),
                2 => (height - h1 + y) * width + (width - w1 + x),
                _ => (height - h1 + y) * width + x,
            };
            shell[idx] = false;
        }
    }
    shell
}

fn bsp_with_walls(shell: &[bool], target_rooms: usize, seed: u64, width: usize, height: usize) -> (Vec<Region>, Vec<bool>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut wall_mask = vec![false; width * height];
    let mut regions = vec![Region::new(shell.to_vec(), width, height)];

    while regions.len() < target_rooms {
        regions.sort_unstable_by_key(|r| usize::MAX - r.area);
        let region = regions.remove(0);
        if region.area < MIN_ROOM_AREA_CELLS * 2 { regions.push(region); break; }
        let (miny, maxy, minx, maxx) = region.bbox;
        if (maxy - miny + 1) < 2 * MIN_THICK_CELLS || (maxx - minx + 1) < 2 * MIN_THICK_CELLS {
            regions.push(region);
            break;
        }
        let mut candidates = Vec::new();
        // Vertical splits
        let mut xs: Vec<usize> = (minx + 1..maxx).collect();
        if xs.len() > SAMPLE_C {
            xs = xs.into_iter().choose_multiple(&mut rng, SAMPLE_C);
        }
        for &x in &xs {
            let col: Vec<bool> = (miny..=maxy).map(|y| region.mask[y * width + x]).collect();
            if !col.iter().all(|&b| b) || find_runs(&col) != 1 { continue; }
            // Area ratio
            let left = region.mask.iter().enumerate().filter(|&(i, &b)| b && (i % width) < x).count();
            let right = region.mask.iter().enumerate().filter(|&(i, &b)| b && (i % width) > x).count();
            if (left as f64) < MIN_SIZE_RATIO * region.area as f64 || (right as f64) < MIN_SIZE_RATIO * region.area as f64 {
                continue;
            }
            // Aspect ratio checks on both sides
            let mut ok = true;
            // Left side
            {
                let side_mask: Vec<bool> = region.mask.iter().enumerate().map(|(i, &b)| b && (i % width) < x).collect();
                let (mw, mh) = thinnest_segment_length(&side_mask, width, height);
                let ys2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i / width) } else { None }).collect();
                let xs2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i % width) } else { None }).collect();
                let bw = xs2.iter().max().unwrap() - xs2.iter().min().unwrap() + 1;
                let bh = ys2.iter().max().unwrap() - ys2.iter().min().unwrap() + 1;
                let ar = if bw >= bh { bw as f64 / mh.max(1) as f64 } else { bh as f64 / mw.max(1) as f64 };
                if ar < MIN_AR || ar > MAX_AR { ok = false; }
            }
            // Right side
            if ok {
                let side_mask: Vec<bool> = region.mask.iter().enumerate().map(|(i, &b)| b && (i % width) > x).collect();
                let (mw, mh) = thinnest_segment_length(&side_mask, width, height);
                let ys2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i / width) } else { None }).collect();
                let xs2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i % width) } else { None }).collect();
                let bw = xs2.iter().max().unwrap() - xs2.iter().min().unwrap() + 1;
                let bh = ys2.iter().max().unwrap() - ys2.iter().min().unwrap() + 1;
                let ar = if bw >= bh { bw as f64 / mh.max(1) as f64 } else { bh as f64 / mw.max(1) as f64 };
                if ar < MIN_AR || ar > MAX_AR { ok = false; }
            }
            if ok { candidates.push(('v', x)); }
        }
        // Horizontal splits
        let mut ys: Vec<usize> = (miny + 1..maxy).collect();
        if ys.len() > SAMPLE_C {
            ys = ys.into_iter().choose_multiple(&mut rng, SAMPLE_C);
        }
        for &y in &ys {
            let row: Vec<bool> = (minx..=maxx).map(|x| region.mask[y * width + x]).collect();
            if !row.iter().all(|&b| b) || find_runs(&row) != 1 { continue; }
            // Area ratio
            let top = region.mask.iter().enumerate().filter(|&(i, &b)| b && (i / width) < y).count();
            let bot = region.mask.iter().enumerate().filter(|&(i, &b)| b && (i / width) > y).count();
            if (top as f64) < MIN_SIZE_RATIO * region.area as f64 || (bot as f64) < MIN_SIZE_RATIO * region.area as f64 {
                continue;
            }
            // Aspect ratio checks on both sides
            let mut ok = true;
            // Top side
            {
                let side_mask: Vec<bool> = region.mask.iter().enumerate().map(|(i, &b)| b && (i / width) < y).collect();
                let (mw, mh) = thinnest_segment_length(&side_mask, width, height);
                let ys2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i / width) } else { None }).collect();
                let xs2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i % width) } else { None }).collect();
                let bw = xs2.iter().max().unwrap() - xs2.iter().min().unwrap() + 1;
                let bh = ys2.iter().max().unwrap() - ys2.iter().min().unwrap() + 1;
                let ar = if bw >= bh { bw as f64 / mh.max(1) as f64 } else { bh as f64 / mw.max(1) as f64 };
                if ar < MIN_AR || ar > MAX_AR { ok = false; }
            }
            // Bottom side
            if ok {
                let side_mask: Vec<bool> = region.mask.iter().enumerate().map(|(i, &b)| b && (i / width) > y).collect();
                let (mw, mh) = thinnest_segment_length(&side_mask, width, height);
                let ys2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i / width) } else { None }).collect();
                let xs2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i % width) } else { None }).collect();
                let bw = xs2.iter().max().unwrap() - xs2.iter().min().unwrap() + 1;
                let bh = ys2.iter().max().unwrap() - ys2.iter().min().unwrap() + 1;
                let ar = if bw >= bh { bw as f64 / mh.max(1) as f64 } else { bh as f64 / mw.max(1) as f64 };
                if ar < MIN_AR || ar > MAX_AR { ok = false; }
            }
            if ok { candidates.push(('h', y)); }
        }
        if candidates.is_empty() {
            regions.push(region);
            break;
        }
        let &(orient, coord) = candidates.iter().choose(&mut rng).unwrap();
        let mut mask_a = region.mask.clone();
        let mut mask_b = region.mask.clone();
        if orient == 'v' {
            for y in miny..=maxy { wall_mask[y * width + coord] = true; }
            for idx in 0..mask_a.len() {
                if idx % width >= coord { mask_a[idx] = false; }
                if idx % width <= coord { mask_b[idx] = false; }
            }
        } else {
            for x in minx..=maxx { wall_mask[coord * width + x] = true; }
            for idx in 0..mask_a.len() {
                if idx / width >= coord { mask_a[idx] = false; }
                if idx / width <= coord { mask_b[idx] = false; }
            }
        }
        if mask_a.iter().filter(|&&b| b).count() >= mask_b.iter().filter(|&&b| b).count() {
            regions.push(Region::new(mask_a, width, height));
            regions.push(Region::new(mask_b, width, height));
        } else {
            regions.push(Region::new(mask_b, width, height));
            regions.push(Region::new(mask_a, width, height));
        }
    }
    // Outer walls (including diagonal)
    for y in 0..height {
        for x in 0..width {
            if !shell[y * width + x] { continue; }
            'outer: for dy in -1..=1 {
                for dx in -1..=1 {
                    if dy == 0 && dx == 0 { continue; }
                    let ny = y as isize + dy;
                    let nx = x as isize + dx;
                    if ny < 0 || nx < 0 || ny >= height as isize || nx >= width as isize || !shell[ny as usize * width + nx as usize] {
                        wall_mask[y * width + x] = true;
                        break 'outer;
                    }
                }
            }
        }
    }
    (regions, wall_mask)
}

fn build_labels(regions: &[Region], width: usize, height: usize) -> Vec<Cell> {
    let mut labels = vec![OUTSIDE; width * height];
    for (rid, reg) in regions.iter().enumerate() {
        for (i, &b) in reg.mask.iter().enumerate() {
            if b { labels[i] = rid as Cell; }
        }
    }
    labels
}

fn carve_doors(labels: &[Cell], wall_mask: &mut [bool], shell: &[bool], seed: u64, width: usize, height: usize) -> Vec<bool> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut adjacency: BTreeMap<(Cell, Cell), Vec<usize>> = BTreeMap::new();
    for y in 1..height-1 {
        for x in 1..width-1 {
            let idx = y * width + x;
            if !wall_mask[idx] { continue; }
            if !(shell[idx - width] && shell[idx + width] && shell[idx - 1] && shell[idx + 1]) { continue; }
            let left = labels[idx - 1]; let right = labels[idx + 1];
            if left >= 0 && right >= 0 && left != right {
                let key = if left < right { (left, right) } else { (right, left) };
                adjacency.entry(key).or_default().push(idx);
                continue;
            }
            let up = labels[idx - width]; let down = labels[idx + width];
            if up >= 0 && down >= 0 && up != down {
                let key = if up < down { (up, down) } else { (down, up) };
                adjacency.entry(key).or_default().push(idx);
            }
        }
    }
    if adjacency.is_empty() {
        return vec![false; width * height];
    }
    let mut graph: BTreeMap<Cell, BTreeSet<Cell>> = BTreeMap::new();
    for (&(a, b), _) in &adjacency {
        graph.entry(a).or_default().insert(b);
        graph.entry(b).or_default().insert(a);
    }
    let central = *graph.keys().max_by_key(|&&k| graph[&k].len()).unwrap();
    let mut seen = BTreeSet::new(); seen.insert(central);
    let mut queue = VecDeque::new(); queue.push_back(central);
    let mut tree_edges = BTreeSet::new();
    while let Some(u) = queue.pop_front() {
        if let Some(neighbors) = graph.get(&u) {
            for &v in neighbors {
                if !seen.contains(&v) {
                    seen.insert(v);
                    tree_edges.insert(if u < v { (u, v) } else { (v, u) });
                    queue.push_back(v);
                }
            }
        }
    }
    let mut door_mask = vec![false; width * height];
    // for each edge ...
    for edge in tree_edges.into_iter() {
        if let Some(mut cells) = adjacency.get(&edge).cloned() {
            let coords: Vec<(usize, usize)> = cells.iter().map(|&i| (i / width, i % width)).collect();
            let all_x_equal = coords.iter().all(|&(_, x)| x == coords[0].1);
            if all_x_equal {
                cells.sort_unstable_by_key(|&i| i / width);
            } else {
                cells.sort_unstable_by_key(|&i| i % width);
            }
            let total = cells.len();
            let width = rng.gen_range(DOOR_MIN_CELLS..=DOOR_MAX_CELLS).min(total);
            let start = rng.gen_range(0..=total - width);
            for &idx in &cells[start..start + width] {
                wall_mask[idx] = false;
                door_mask[idx] = true;
            }
        }
    }
    door_mask
}

/// Simulation world bundling layout and objects
#[derive(Debug)]
pub struct World {
    pub layout: Layout,
    pub objects: Vec<Object>,
}

/// Randomly place objects according to schemas
fn place_objects(layout: &Layout, schemas: &[ObjectSchema], seed: u64, max_objects: usize) -> Vec<Object> {
    let mut rng = StdRng::seed_from_u64(seed);
    use rand::seq::{SliceRandom, IteratorRandom};
    let mut objects: Vec<Object> = Vec::new();
    let mut id = 0;
    // collect all room cells
    let mut free_cells: Vec<(usize, usize)> = (0..layout.height).flat_map(|y| {
        (0..layout.width).filter_map(move |x| {
            let idx = y * layout.width + x;
            if layout.cells[idx] >= 0 { Some((x, y)) } else { None }
        })
    }).collect();
    free_cells.shuffle(&mut rng);
    // shuffle placement order of schemas
    let mut schema_list: Vec<&ObjectSchema> = schemas.iter().collect();
    schema_list.shuffle(&mut rng);
    for schema in schema_list.iter() {
        if id >= max_objects { break; }
        match &schema.constraint {
            // must go inside specific parent(s)
            ObjectConstraint::InsideOf(names) => {
                // find matching parent indices
                let parent_indices: Vec<usize> = objects.iter().enumerate()
                    .filter(|(_, o)| names.contains(&o.name) && o.contents.len() < o.capacity)
                    .map(|(i, _)| i)
                    .collect();
                if let Some(&pi) = parent_indices.iter().choose(&mut rng) {
                    // place child inside parent
                    let (px, py) = (objects[pi].x, objects[pi].y);
                    objects[pi].contents.push(id);
                    objects.push(Object { id,
                                          name: schema.name,
                                          capacity: schema.capacity,
                                          pickable: schema.pickable,
                                          x: px,
                                          y: py,
                                          contents: Vec::new() });
                    id += 1;
                }
            }
            _ => {
                // non-InsideOf: determine floor vs parent if pickable
                let mut placed = false;
                if schema.pickable {
                    // try any parent by selecting indices
                    let parent_indices: Vec<usize> = objects.iter().enumerate()
                        .filter(|(_, o)| o.contents.len() < o.capacity)
                        .map(|(i, _)| i)
                        .collect();
                    if !parent_indices.is_empty() && rng.gen_bool(0.5) {
                        if let Some(&pi) = parent_indices.iter().choose(&mut rng) {
                            let (px, py) = (objects[pi].x, objects[pi].y);
                            objects[pi].contents.push(id);
                            objects.push(Object { id,
                                                  name: schema.name,
                                                  capacity: schema.capacity,
                                                  pickable: schema.pickable,
                                                  x: px,
                                                  y: py,
                                                  contents: Vec::new() });
                            id += 1;
                            placed = true;
                        }
                    }
                }
                // if not placed in parent, place on floor under its constraint
                if !placed {
                    let candidates: Vec<(usize, usize)> = free_cells.iter().cloned()
                        .filter(|&(x, y)| schema.constraint.check(layout, x, y))
                        .collect();
                    if let Some(&(fx, fy)) = candidates.iter().choose(&mut rng) {
                        objects.push(Object { id,
                                              name: schema.name,
                                              capacity: schema.capacity,
                                              pickable: schema.pickable,
                                              x: fx,
                                              y: fy,
                                              contents: Vec::new() });
                        id += 1;
                    }
                }
            }
        }
    }
    // enforce minimum 25% pickable and 25% capacity-bearing items
    let total = objects.len();
    let mut needed_pick = ((total as f32) * 0.25).ceil() as usize;
    let cur_pick = objects.iter().filter(|o| o.pickable).count();
    if cur_pick < needed_pick {
        needed_pick -= cur_pick;
        for _ in 0..needed_pick {
            // random pickable schema (non-InsideOf)
            if let Some(schema) = schemas.iter()
                .filter(|s| s.pickable)
                .filter(|s| !matches!(s.constraint, ObjectConstraint::InsideOf(_)))
                .choose(&mut rng)
            {
                // floor placement
                let candidates: Vec<(usize, usize)> = free_cells.iter().cloned()
                    .filter(|&(x, y)| schema.constraint.check(layout, x, y))
                    .collect();
                if let Some(&(x, y)) = candidates.iter().choose(&mut rng) {
                    objects.push(Object { id,
                                          name: schema.name,
                                          capacity: schema.capacity,
                                          pickable: schema.pickable,
                                          x, y,
                                          contents: Vec::new() });
                    id += 1;
                }
            }
        }
    }
    let total = objects.len();
    let mut needed_cap = ((total as f32) * 0.25).ceil() as usize;
    let cur_cap = objects.iter().filter(|o| o.capacity > 0).count();
    if cur_cap < needed_cap {
        needed_cap -= cur_cap;
        for _ in 0..needed_cap {
            // random capacity schema
            if let Some(schema) = schemas.iter()
                .filter(|s| s.capacity > 0)
                .filter(|s| !matches!(s.constraint, ObjectConstraint::InsideOf(_)))
                .choose(&mut rng)
            {
                let candidates: Vec<(usize, usize)> = free_cells.iter().cloned()
                    .filter(|&(x, y)| schema.constraint.check(layout, x, y))
                    .collect();
                if let Some(&(x, y)) = candidates.iter().choose(&mut rng) {
                    objects.push(Object { id,
                                          name: schema.name,
                                          capacity: schema.capacity,
                                          pickable: schema.pickable,
                                          x, y,
                                          contents: Vec::new() });
                    id += 1;
                }
            }
        }
    }
    objects
}

/// Generate a new world with dynamic width/height and object placement
pub fn generate(opts: &GenOpts) -> World {
    let width = opts.width;
    let height = opts.height;
    let shell = make_concave_shell(width, height, opts.seed);
    let (regions, mut wall_mask) = bsp_with_walls(&shell, opts.max_rooms, opts.seed, width, height);
    let labels = build_labels(&regions, width, height);
    let door_mask = carve_doors(&labels, &mut wall_mask, &shell, opts.seed, width, height);
    let mut cells = Vec::with_capacity(width * height);
    for i in 0..width * height {
        if !shell[i] {
            cells.push(OUTSIDE);
        } else if wall_mask[i] {
            cells.push(WALL);
        } else if door_mask[i] {
            cells.push(CLOSED_DOOR);
        } else {
            cells.push(labels[i]);
        }
    }
    let layout = Layout::new(width, height, cells);
    let objects = place_objects(&layout, &ObjectSchema::default_schemas(), opts.seed, opts.max_objects);
    World { layout, objects }
}