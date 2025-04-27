<script setup>
import { ref, watch, onMounted, nextTick, onBeforeUnmount } from 'vue';
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
    // door opening with WASD
    else if (e.key === 'w') sim.value.open_up();
    else if (e.key === 's') sim.value.open_down();
    else if (e.key === 'a') sim.value.open_left();
    else if (e.key === 'd') sim.value.open_right();
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
  <div style="position:absolute; top:10px; left:10px; background:rgba(0,0,0,0.7); color:#fff; padding:12px; border-radius:6px;">
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

  <canvas ref="canvasRef" style="display:block; background:#ddd; margin-top:0"></canvas>
</template>

<style>
html, body, #app { margin:0; padding:0; }
</style>
