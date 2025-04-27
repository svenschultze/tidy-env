<script setup>
import { ref, watch, onMounted, nextTick, onBeforeUnmount, computed } from 'vue';
import initWasm, { generate, ApartmentSimulator } from './pkg/wasm.js';
// simulator instance
const sim = ref(null);
// user-configurable parameters and canvas reference
const seed = ref(0);
const maxRooms = ref(4);
const canvasRef = ref(null);
const scale = 10;
const margin = 2;

// room color cache and helper
const roomColors = {};
function colorFor(v) {
  if (v < 0) {
    switch (v) {
      case -1: return '#fff'; // wall
      case -2: return '#000'; // outside
      case -3: return '#ccc'; // closed door
      case -4: return '#999'; // open door
      default: return '#f00'; // unknown
    }
  }
  if (!(v in roomColors)) {
    const hue = (v * 73) % 360;
    roomColors[v] = `hsl(${hue},60%,70%)`;
  }
  return roomColors[v];
}

// map object types to colors
const objectColors = {
  Wardrobe: '#795548',
  Cupboard: '#8D6E63',
  Banana: '#FFEB3B',
  Couch: '#4CAF50',
  Unknown: '#888',
};

// compute unique types for legend
const legendItems = computed(() => {
  if (!sim.value) return [];
  const types = new Set();
  try {
    const objs = sim.value.get_objects();
    for (const o of objs) types.add(o.type);
  } catch {}
  return Array.from(types);
});

// initialize simulator and first render
async function initSim() {
  await initWasm();
  const layout = generate(BigInt(seed.value), maxRooms.value);
  const freeIdx = layout.cells.findIndex(v => v >= 0);
  const startX = freeIdx % layout.width;
  const startY = Math.floor(freeIdx / layout.width);
  sim.value = new ApartmentSimulator(BigInt(seed.value), maxRooms.value, startX, startY);
  drawWorld();
}

// draw current simulation state
function drawWorld() {
  if (!sim.value) return;
  const W = sim.value.width;
  const H = sim.value.height;
  const cells = sim.value.cells;

  // resize canvas
  const canvas = canvasRef.value;
  canvas.width  = W*scale + margin*2;
  canvas.height = H*scale + margin*2;
  const ctx = canvas.getContext('2d');

  // clear
  ctx.fillStyle = '#ccc';
  ctx.fillRect(0,0,canvas.width,canvas.height);

  // reset room color cache
  for (let k in roomColors) delete roomColors[k];

  // draw each cell
  for (let i = 0; i < cells.length; i++) {
    const v = cells[i];
    const x = i % W;
    const y = Math.floor(i / W);
    ctx.fillStyle = colorFor(v);
    ctx.fillRect(
      margin + x*scale,
      margin + y*scale,
      scale,
      scale
    );
  }

  // draw objects (before agent)
  try {
    const objs = sim.value.get_objects();
    console.log(objs);
    for (const o of objs) {
      const color = objectColors[o.type] || (o.pickable ? '#FFEB3B' : '#888');
      ctx.fillStyle = color;
      const ox = o.x, oy = o.y;
      const size = scale * 0.6;
      ctx.fillRect(
        margin + ox*scale + (scale - size)/2,
        margin + oy*scale + (scale - size)/2,
        size,
        size
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

// handle keyboard
function handleKey(e) {
  if (!sim.value) return;
  try {
    // movement
    if (e.key === 'ArrowUp') sim.value.up();
    else if (e.key === 'ArrowDown') sim.value.down();
    else if (e.key === 'ArrowLeft') sim.value.left();
    else if (e.key === 'ArrowRight') sim.value.right();
    // door and object interaction in directions
    else if (e.key === 'w') sim.value.interact(0, -1);
    else if (e.key === 's') sim.value.interact(0, 1);
    else if (e.key === 'a') sim.value.interact(-1, 0);
    else if (e.key === 'd') sim.value.interact(1, 0);
    // object interaction: pick up or drop/place via unified interact
    else if (e.key === 'e' || e.key === 'q') sim.value.interact(0, 0);
    else return;
    drawWorld();
  } catch (err) {
    console.error(err);
  }
}

watch([seed, maxRooms], () => {
  nextTick().then(initSim);
});
onMounted(() => {
  nextTick().then(initSim);
  window.addEventListener('keydown', handleKey);
});
onBeforeUnmount(() => window.removeEventListener('keydown', handleKey));
</script>

<template>
  <div style="display:flex;">

    <canvas ref="canvasRef"/>
    <div id="legend">
      <div>
        <h2>Legend</h2>
        <div>
          <!-- control explanation -->
           <p>
            <strong>Arrow keys:</strong> Move
            <br/>
            <strong>WASD:</strong> Interact with doors and objects
          </p>
        </div>
        <div v-for="type in legendItems" :key="type" style="display:flex; align-items:center; margin-bottom:4px;">
          <div :style="{width:'16px',height:'16px',backgroundColor: objectColors[type] || '#888', marginRight:'8px'}"></div>
          <span>{{ type }}</span>
        </div>
      </div>
      
      <div>
        <div>
          <label>Seed:
            <input type="number" v-model="seed" style="width:80px"/>
          </label>
        </div>
        <div style="margin-top:4px">
          <label>Rooms:
            <input type="number" min="1" max="12" v-model="maxRooms" style="width:50px"/>
          </label>
        </div>
        <button style="margin-top:8px" @click="initSim">Regenerate</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
canvas {
  flex: 1;
}
#legend {
  background:rgba(0,0,0,0.7);
  color:#fff;
  padding:12px;
  border-radius:6px;
  margin-left:12px;
  display:flex;
  flex-direction:column;
  justify-content:space-between;
}
h2, h3 {
  margin-top: 0;
  margin-bottom: 4px;
}
</style>
