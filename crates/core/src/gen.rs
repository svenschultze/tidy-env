use rand::{Rng, SeedableRng};
use rand::seq::IteratorRandom;
use rand::rngs::StdRng;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    OUTSIDE, WALL, CLOSED_DOOR
};

pub type Cell = i8;

#[derive(Clone, Copy)]
pub struct GenOpts {
    pub seed: u64,
    pub max_rooms: usize,
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

// Parameters
const WIDTH: usize = 30;
const DEPTH: usize = 20;
const GRID: f64 = 0.5;
const RES: usize = (1.0 / GRID) as usize;
const W: usize = WIDTH * RES;
const H: usize = DEPTH * RES;

const MIN_THICK_CELLS: usize = 3;            // ceil(1.2 / 0.5)
const MIN_ROOM_AREA_CELLS: usize = 24;        // ceil(6.0 / (0.5*0.5))

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

#[derive(Clone)]
struct Region {
    mask: Vec<bool>,
    area: usize,
    bbox: (usize, usize, usize, usize),
}

impl Region {
    fn new(mask: Vec<bool>) -> Self {
        let mut ys = Vec::new();
        let mut xs = Vec::new();
        for y in 0..H {
            for x in 0..W {
                if mask[y * W + x] {
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

fn make_concave_shell(seed: u64) -> Vec<bool> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut shell = vec![true; W * H];
    let w1 = rng.gen_range((3.0/GRID) as usize..=(5.0/GRID) as usize);
    let h1 = rng.gen_range((3.0/GRID) as usize..=(5.0/GRID) as usize);
    let corner = rng.gen_range(0..4);
    for y in 0..h1 {
        for x in 0..w1 {
            let idx = match corner {
                0 => y * W + x,
                1 => y * W + (W - w1 + x),
                2 => (H - h1 + y) * W + (W - w1 + x),
                _ => (H - h1 + y) * W + x,
            };
            shell[idx] = false;
        }
    }
    shell
}

fn bsp_with_walls(shell: &[bool], target_rooms: usize, seed: u64) -> (Vec<Region>, Vec<bool>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut wall_mask = vec![false; W * H];
    let mut regions = vec![Region::new(shell.to_vec())];

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
            let col: Vec<bool> = (miny..=maxy).map(|y| region.mask[y * W + x]).collect();
            if !col.iter().all(|&b| b) || find_runs(&col) != 1 { continue; }
            // Area ratio
            let left = region.mask.iter().enumerate().filter(|&(i, &b)| b && (i % W) < x).count();
            let right = region.mask.iter().enumerate().filter(|&(i, &b)| b && (i % W) > x).count();
            if (left as f64) < MIN_SIZE_RATIO * region.area as f64 || (right as f64) < MIN_SIZE_RATIO * region.area as f64 {
                continue;
            }
            // Aspect ratio checks on both sides
            let mut ok = true;
            // Left side
            {
                let side_mask: Vec<bool> = region.mask.iter().enumerate().map(|(i, &b)| b && (i % W) < x).collect();
                let (mw, mh) = thinnest_segment_length(&side_mask, W, H);
                let ys2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i / W) } else { None }).collect();
                let xs2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i % W) } else { None }).collect();
                let bw = xs2.iter().max().unwrap() - xs2.iter().min().unwrap() + 1;
                let bh = ys2.iter().max().unwrap() - ys2.iter().min().unwrap() + 1;
                let ar = if bw >= bh { bw as f64 / mh.max(1) as f64 } else { bh as f64 / mw.max(1) as f64 };
                if ar < MIN_AR || ar > MAX_AR { ok = false; }
            }
            // Right side
            if ok {
                let side_mask: Vec<bool> = region.mask.iter().enumerate().map(|(i, &b)| b && (i % W) > x).collect();
                let (mw, mh) = thinnest_segment_length(&side_mask, W, H);
                let ys2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i / W) } else { None }).collect();
                let xs2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i % W) } else { None }).collect();
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
            let row: Vec<bool> = (minx..=maxx).map(|x| region.mask[y * W + x]).collect();
            if !row.iter().all(|&b| b) || find_runs(&row) != 1 { continue; }
            // Area ratio
            let top = region.mask.iter().enumerate().filter(|&(i, &b)| b && (i / W) < y).count();
            let bot = region.mask.iter().enumerate().filter(|&(i, &b)| b && (i / W) > y).count();
            if (top as f64) < MIN_SIZE_RATIO * region.area as f64 || (bot as f64) < MIN_SIZE_RATIO * region.area as f64 {
                continue;
            }
            // Aspect ratio checks on both sides
            let mut ok = true;
            // Top side
            {
                let side_mask: Vec<bool> = region.mask.iter().enumerate().map(|(i, &b)| b && (i / W) < y).collect();
                let (mw, mh) = thinnest_segment_length(&side_mask, W, H);
                let ys2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i / W) } else { None }).collect();
                let xs2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i % W) } else { None }).collect();
                let bw = xs2.iter().max().unwrap() - xs2.iter().min().unwrap() + 1;
                let bh = ys2.iter().max().unwrap() - ys2.iter().min().unwrap() + 1;
                let ar = if bw >= bh { bw as f64 / mh.max(1) as f64 } else { bh as f64 / mw.max(1) as f64 };
                if ar < MIN_AR || ar > MAX_AR { ok = false; }
            }
            // Bottom side
            if ok {
                let side_mask: Vec<bool> = region.mask.iter().enumerate().map(|(i, &b)| b && (i / W) > y).collect();
                let (mw, mh) = thinnest_segment_length(&side_mask, W, H);
                let ys2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i / W) } else { None }).collect();
                let xs2: Vec<usize> = side_mask.iter().enumerate().filter_map(|(i, &b)| if b { Some(i % W) } else { None }).collect();
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
            for y in miny..=maxy { wall_mask[y * W + coord] = true; }
            for idx in 0..mask_a.len() {
                if idx % W >= coord { mask_a[idx] = false; }
                if idx % W <= coord { mask_b[idx] = false; }
            }
        } else {
            for x in minx..=maxx { wall_mask[coord * W + x] = true; }
            for idx in 0..mask_a.len() {
                if idx / W >= coord { mask_a[idx] = false; }
                if idx / W <= coord { mask_b[idx] = false; }
            }
        }
        if mask_a.iter().filter(|&&b| b).count() >= mask_b.iter().filter(|&&b| b).count() {
            regions.push(Region::new(mask_a));
            regions.push(Region::new(mask_b));
        } else {
            regions.push(Region::new(mask_b));
            regions.push(Region::new(mask_a));
        }
    }
    // Outer walls (including diagonal)
    for y in 0..H {
        for x in 0..W {
            if !shell[y * W + x] { continue; }
            'outer: for dy in -1..=1 {
                for dx in -1..=1 {
                    if dy == 0 && dx == 0 { continue; }
                    let ny = y as isize + dy;
                    let nx = x as isize + dx;
                    if ny < 0 || nx < 0 || ny >= H as isize || nx >= W as isize || !shell[ny as usize * W + nx as usize] {
                        wall_mask[y * W + x] = true;
                        break 'outer;
                    }
                }
            }
        }
    }
    (regions, wall_mask)
}

fn build_labels(regions: &[Region]) -> Vec<Cell> {
    let mut labels = vec![OUTSIDE; W * H];
    for (rid, reg) in regions.iter().enumerate() {
        for (i, &b) in reg.mask.iter().enumerate() {
            if b { labels[i] = rid as Cell; }
        }
    }
    labels
}

fn carve_doors(labels: &[Cell], wall_mask: &mut [bool], shell: &[bool], seed: u64) -> Vec<bool> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut adjacency: BTreeMap<(Cell, Cell), Vec<usize>> = BTreeMap::new();
    for y in 1..H-1 {
        for x in 1..W-1 {
            let idx = y * W + x;
            if !wall_mask[idx] { continue; }
            if !(shell[idx - W] && shell[idx + W] && shell[idx - 1] && shell[idx + 1]) { continue; }
            let left = labels[idx - 1]; let right = labels[idx + 1];
            if left >= 0 && right >= 0 && left != right {
                let key = if left < right { (left, right) } else { (right, left) };
                adjacency.entry(key).or_default().push(idx);
                continue;
            }
            let up = labels[idx - W]; let down = labels[idx + W];
            if up >= 0 && down >= 0 && up != down {
                let key = if up < down { (up, down) } else { (down, up) };
                adjacency.entry(key).or_default().push(idx);
            }
        }
    }
    if adjacency.is_empty() {
        return vec![false; W * H];
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
    let mut door_mask = vec![false; W * H];
    for edge in tree_edges.into_iter() {
        if let Some(mut cells) = adjacency.get(&edge).cloned() {
            let coords: Vec<(usize, usize)> = cells.iter().map(|&i| (i / W, i % W)).collect();
            let all_x_equal = coords.iter().all(|&(_, x)| x == coords[0].1);
            if all_x_equal {
                cells.sort_unstable_by_key(|&i| i / W);
            } else {
                cells.sort_unstable_by_key(|&i| i % W);
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

pub fn generate(opts: &GenOpts) -> Layout {
    let shell = make_concave_shell(opts.seed);
    let (regions, mut wall_mask) = bsp_with_walls(&shell, opts.max_rooms, opts.seed);
    let labels = build_labels(&regions);
    let door_mask = carve_doors(&labels, &mut wall_mask, &shell, opts.seed);
    let mut cells = Vec::with_capacity(W * H);
    for i in 0..W * H {
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
    Layout::new(W, H, cells)
}