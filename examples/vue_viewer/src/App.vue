<script setup>
import { ref, watch, onMounted, nextTick, onBeforeUnmount, computed } from 'vue';
import initWasm, { ApartmentSimulator } from './pkg/wasm.js';

// simulator instance
const sim = ref(null);
// user-configurable parameters and canvas reference
const seed = ref(0);
const maxRooms = ref(4);
const width = ref(30);
const height = ref(20);
const max_objects = ref(10);
const canvasRef = ref(null);
const scale = 100;
const margin = 2;

// color cache for rooms
const roomColors = {};
function colorFor(v) {
  if (v < 0) {
    switch (v) {
      case -1: return '#fff'; // wall
      case -2: return '#000'; // outside
      case -3: return '#999'; // closed door
      case -4: return '#ccc'; // open door
      default: return '#f00'; // unknown
    }
  }
  if (!(v in roomColors)) {
    const hue = (v * 73) % 360;
    roomColors[v] = `hsl(${hue},60%,70%)`;
  }
  return roomColors[v];
}

// unique color per object instance
const objectColors = {};
function colorForObject(id) {
  if (!(id in objectColors)) {
    const hue = (id * 137) % 360; // spread hues
    objectColors[id] = `hsl(${hue},60%,50%)`;
  }
  return objectColors[id];
}

// trigger legend update after interactions
const updateCount = ref(0);

// compute objects for legend (instance-level)
const legendItems = computed(() => {
  updateCount.value;
  if (!sim.value) return [];
  return sim.value.get_objects() || [];
});

// agent holding
const holding = computed(() => {
  updateCount.value;
  if (!sim.value) return null;
  const h = sim.value.get_holding();
  return h || null;
});

// containers
const containers = computed(() => {
  updateCount.value;
  if (!sim.value) return [];
  return sim.value.get_objects().filter(o => o.capacity > 0);
});

// get contents of container
function getContents(id) {
  if (!sim.value) return [];
  return sim.value.get_contents(id) || [];
}

// init and draw
async function initSim() {
  await initWasm();
  sim.value = new ApartmentSimulator(
    BigInt(seed.value), maxRooms.value,
    width.value, height.value,
    max_objects.value,
  );
  drawWorld();
}

function drawWorld() {
  if (!sim.value) return;
  const W = sim.value.width;
  const H = sim.value.height;
  const cells = sim.value.cells;
  const canvas = canvasRef.value;
  canvas.width = W*scale + margin*2;
  canvas.height = H*scale + margin*2;
  const ctx = canvas.getContext('2d');

  // clear background
  ctx.fillStyle = '#ccc';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  // reset room colors
  for (let k in roomColors) delete roomColors[k];

  // draw grid cells
  for (let i = 0; i < cells.length; i++) {
    const v = cells[i];
    const x = i % W;
    const y = Math.floor(i / W);
    ctx.fillStyle = colorFor(v);
    ctx.fillRect(margin + x*scale, margin + y*scale, scale, scale);
  }

  // draw objects
  try {
    const objs = sim.value.get_objects();
    for (const o of objs) {
      const ox = o.x, oy = o.y;
      const size = scale * 0.6;
      ctx.fillStyle = colorForObject(o.id);
      ctx.fillRect(
        margin + ox*scale + (scale-size)/2,
        margin + oy*scale + (scale-size)/2,
        size,
        size
      );
      // draw object ID number
      ctx.fillStyle = '#000';
      ctx.font = `${scale*0.5}px sans-serif`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(
        String(o.id),
        margin + ox*scale + scale/2,
        margin + oy*scale + scale/2
      );
    }
  } catch (err) {
    console.error('Error drawing objects', err);
  }

  // draw agent
  ctx.fillStyle = '#f00';
  const ax = sim.value.agent_x;
  const ay = sim.value.agent_y;
  ctx.beginPath();
  ctx.arc(
    margin + ax*scale + scale/2,
    margin + ay*scale + scale/2,
    scale/3,
    0,
    2*Math.PI
  );
  ctx.fill();
}

// keyboard handling
function handleKey(e) {
  if (!sim.value) return;
  const moves = {
    ArrowUp: () => sim.value.up(),
    ArrowDown: () => sim.value.down(),
    ArrowLeft: () => sim.value.left(),
    ArrowRight: () => sim.value.right(),
  };
  const interacts = {
    w: () => sim.value.interact(0, -1),
    s: () => sim.value.interact(0, 1),
    a: () => sim.value.interact(-1, 0),
    d: () => sim.value.interact(1, 0),
  };
  if (moves[e.key]) moves[e.key]();
  else if (interacts[e.key]) interacts[e.key]();
  else return;
  drawWorld();
  updateCount.value++;
}

watch([seed, maxRooms, width, height, max_objects], () => nextTick().then(initSim));
onMounted(() => {
  nextTick().then(initSim);
  window.addEventListener('keydown', handleKey);
});
onBeforeUnmount(() => window.removeEventListener('keydown', handleKey));
</script>

<template>
  <div class="layout">
    <div class="canvas-container">
      <canvas ref="canvasRef" class="canvas" />
    </div>
     <div id="legend" class="legend">
      <div class="legend-content">
        <h2>Legend</h2>
        <div>
          <p>
            <strong>Arrow keys:</strong> Move<br />
            <strong>WASD:</strong> Interact
          </p>
        </div>
        <!-- Object legend per instance -->
        <div v-for="obj in legendItems" :key="obj.id" class="legend-item">
          <div class="legend-color" :style="{ backgroundColor: colorForObject(obj.id) }" />
          <span>{{ obj.id }}: {{ obj.name }}</span>
        </div>

        <div>
          <h3>Agent Holding</h3>
          <div v-if="holding" class="legend-item holding-item">
            <div class="legend-color" :style="{ backgroundColor: colorForObject(holding.id) }" />
            <span>{{ holding.name }}</span>
          </div>
          <div v-else><em>None</em></div>
        </div>

        <div>
          <h3>Container Contents</h3>
          <div v-for="c in containers" :key="c.id" class="container-item">
            <div class="legend-item">
              <div class="legend-color" :style="{ backgroundColor: colorForObject(c.id) }" />
              <strong>{{ c.name }} {{ c.id }}:</strong>
              <span v-if="getContents(c.id).length" class="container-contents">
                <span v-for="o in getContents(c.id)" :key="o.id" class="container-object">
                  {{ o.name }} ({{ o.id }})
                </span>
              </span>
              <span v-else class="container-empty"><em>Empty</em></span>
            </div>
          </div>
        </div>
      </div>

      <div class="controls">
        <div class="control-row">
          <label>Seed: <input type="number" v-model="seed" class="seed-input" /></label>
        </div>
        <div class="control-row">
          <label>Rooms: <input type="number" min="1" max="12" v-model="maxRooms" class="rooms-input" /></label>
        </div>
        <div class="control-row">
          <label>Width: <input type="number" min="10" max="50" v-model="width" class="size-input" /></label>
        </div>
        <div class="control-row">
          <label>Height: <input type="number" min="10" max="50" v-model="height" class="size-input" /></label>
        </div>
        <div class="control-row">
          <label>Max Objects: <input type="number" min="1" max="100" v-model="max_objects" class="size-input" /></label>
        </div>
        <button class="regenerate-button" @click="initSim">Regenerate</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
}

.canvas-container {
  flex: 1;
  image-rendering: pixelated;
}
.legend {
  background: var(--legend-bg);
  color: var(--legend-color);
  padding: var(--legend-padding);
  border-radius: var(--legend-radius);
  margin-left: var(--legend-padding);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}
.legend-item,
.container-item {
  display: flex;
  align-items: center;
  margin-bottom: var(--item-gap);
}
.legend-color {
  width: var(--color-box-size);
  height: var(--color-box-size);
  margin-right: var(--double-gap);
}
.container-object {
  margin-right: var(--item-gap);
}
.container-contents,
.container-empty {
  margin-left: var(--item-gap);
}
.controls {
  margin-top: var(--control-gap);
}
.control-row + .control-row {
  margin-top: var(--item-gap);
}
.seed-input { width: var(--seed-width); }
.rooms-input,
.size-input { width: var(--size-width); }
.regenerate-button { margin-top: var(--control-gap); }
h2, h3 { margin: 0 0 var(--item-gap); }
</style>
