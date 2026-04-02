import init, { WasmGame, heat_color_js } from './pkg/chess_skrolli_problem.js';

await init();

// ── DOM refs ──────────────────────────────────────────────────────────────

const canvas      = document.getElementById('board');
const ctx         = canvas.getContext('2d');
const legendCv    = document.getElementById('legend-canvas');
const legendCtx   = legendCv.getContext('2d');
const statsEl     = document.getElementById('stats');
const sizeInput   = document.getElementById('size');
const itersInput  = document.getElementById('iters');
const speedInput  = document.getElementById('speed');
const speedVal    = document.getElementById('speed-val');
const btnAnimate  = document.getElementById('btn-animate');
const btnHeatmap  = document.getElementById('btn-heatmap');
const btnStop     = document.getElementById('btn-stop');

// ── State ─────────────────────────────────────────────────────────────────

let game = null;
let animId = null;
let lastTs = 0;
let mode = 'idle'; // 'animate' | 'heatmap' | 'idle'

// ── Legend ────────────────────────────────────────────────────────────────

(function drawLegend() {
  const w = legendCv.width;
  for (let i = 0; i < w; i++) {
    const rgb = heat_color_js(i / (w - 1));
    legendCtx.fillStyle = `rgb(${rgb[0]},${rgb[1]},${rgb[2]})`;
    legendCtx.fillRect(i, 0, 1, legendCv.height);
  }
})();

// ── Speed slider ──────────────────────────────────────────────────────────

speedInput.addEventListener('input', () => {
  speedVal.textContent = speedInput.value;
});

// ── Canvas sizing ─────────────────────────────────────────────────────────

const MAX_CANVAS = 600;

function cellSize(n) {
  return Math.max(4, Math.floor(MAX_CANVAS / n));
}

function resizeCanvas(n) {
  const cs = cellSize(n);
  canvas.width  = cs * n;
  canvas.height = cs * n;
}

// ── Drawing ───────────────────────────────────────────────────────────────

// Board y coords: y=0 is bottom of board → canvas row (n-1)
function canvasRow(y, n) { return n - 1 - y; }

function drawBoard(n) {
  const cs = cellSize(n);
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const cx = game.current_x();
  const cy = game.current_y();
  const winX = n - 1, winY = n - 1;
  const loseX = n - 1, loseY = 0;

  for (let x = 0; x < n; x++) {
    for (let y = 0; y < n; y++) {
      const px = x * cs;
      const py = canvasRow(y, n) * cs;

      // Background
      const isLight = (x + y) % 2 === 0;
      ctx.fillStyle = isLight ? '#1e2a3a' : '#16202e';
      ctx.fillRect(px, py, cs, cs);

      // Special cells
      if (x === winX && y === winY) {
        ctx.fillStyle = 'rgba(0,200,100,0.25)';
        ctx.fillRect(px, py, cs, cs);
      } else if (x === loseX && y === loseY) {
        ctx.fillStyle = 'rgba(220,50,50,0.25)';
        ctx.fillRect(px, py, cs, cs);
      } else if (x === 0 && y === 0) {
        ctx.fillStyle = 'rgba(100,100,200,0.15)';
        ctx.fillRect(px, py, cs, cs);
      }

      // Text labels for large enough cells
      if (cs >= 20) {
        ctx.font = `${Math.floor(cs * 0.55)}px serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        const mx = px + cs / 2;
        const my = py + cs / 2;

        if (x === cx && y === cy) {
          ctx.fillStyle = '#ffd166';
          ctx.fillText('♞', mx, my);
        } else if (x === winX && y === winY) {
          ctx.fillStyle = '#06d6a0';
          ctx.fillText('★', mx, my);
        } else if (x === loseX && y === loseY) {
          ctx.fillStyle = '#ef476f';
          ctx.fillText('✗', mx, my);
        } else if (x === 0 && y === 0) {
          ctx.fillStyle = '#888';
          ctx.font = `${Math.floor(cs * 0.4)}px monospace`;
          ctx.fillText('S', mx, my);
        }
      } else {
        // Compact: colored dot for knight/win/lose
        if (x === cx && y === cy) {
          ctx.fillStyle = '#ffd166';
          ctx.fillRect(px + 1, py + 1, cs - 2, cs - 2);
        } else if (x === winX && y === winY) {
          ctx.fillStyle = '#06d6a0';
          ctx.fillRect(px + 1, py + 1, cs - 2, cs - 2);
        } else if (x === loseX && y === loseY) {
          ctx.fillStyle = '#ef476f';
          ctx.fillRect(px + 1, py + 1, cs - 2, cs - 2);
        }
      }
    }
  }
}

function drawHeatmap(n) {
  const cs = cellSize(n);
  const heatmap = game.get_heatmap_normalized(); // Float64Array, index = x*n+y

  ctx.clearRect(0, 0, canvas.width, canvas.height);

  for (let x = 0; x < n; x++) {
    for (let y = 0; y < n; y++) {
      const ratio = heatmap[x * n + y];
      const rgb = heat_color_js(ratio);
      ctx.fillStyle = `rgb(${rgb[0]},${rgb[1]},${rgb[2]})`;
      const px = x * cs;
      const py = canvasRow(y, n) * cs;
      ctx.fillRect(px, py, cs, cs);
    }
  }

  // Mark win/lose/start
  if (cs >= 10) {
    ctx.font = `${Math.floor(cs * 0.55)}px serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const mark = (x, y, sym, color) => {
      const px = x * cs + cs / 2;
      const py = canvasRow(y, n) * cs + cs / 2;
      ctx.fillStyle = '#000';
      ctx.fillText(sym, px + 1, py + 1);
      ctx.fillStyle = color;
      ctx.fillText(sym, px, py);
    };
    mark(n - 1, n - 1, '★', '#fff');
    mark(n - 1, 0,     '✗', '#fff');
    mark(0,     0,     'S', '#ddd');
  }
}

// ── Animation loop ────────────────────────────────────────────────────────

function stopAnimation() {
  if (animId !== null) {
    cancelAnimationFrame(animId);
    animId = null;
  }
  mode = 'idle';
}

function animate(ts) {
  const speed = parseInt(speedInput.value, 10);

  if (ts - lastTs >= speed) {
    lastTs = ts;
    const n = game.board_size();
    const status = game.step(); // 0=ongoing, 1=win, 2=loss

    drawBoard(n);

    if (status === 1) {
      statsEl.innerHTML = `<span style="color:#06d6a0">WIN ★</span> after ${game.move_num()} moves`;
      stopAnimation();
      return;
    } else if (status === 2) {
      statsEl.innerHTML = `<span style="color:#ef476f">LOSE ✗</span> after ${game.move_num()} moves`;
      stopAnimation();
      return;
    } else {
      statsEl.textContent = `Move: ${game.move_num()}`;
    }
  }

  animId = requestAnimationFrame(animate);
}

// ── Button handlers ───────────────────────────────────────────────────────

btnAnimate.addEventListener('click', () => {
  stopAnimation();
  const n = parseInt(sizeInput.value, 10);
  if (isNaN(n) || n < 3 || n > 100) {
    statsEl.textContent = 'Board size must be 3–100.';
    return;
  }
  resizeCanvas(n);
  game = new WasmGame(n);
  mode = 'animate';
  statsEl.textContent = 'Move: 0';
  drawBoard(n);
  lastTs = 0;
  animId = requestAnimationFrame(animate);
});

btnHeatmap.addEventListener('click', () => {
  stopAnimation();
  const n = parseInt(sizeInput.value, 10);
  const iters = parseInt(itersInput.value, 10);
  if (isNaN(n) || n < 3 || n > 100) {
    statsEl.textContent = 'Board size must be 3–100.';
    return;
  }
  if (isNaN(iters) || iters < 1) {
    statsEl.textContent = 'Iterations must be a positive number.';
    return;
  }
  resizeCanvas(n);
  game = new WasmGame(n);
  mode = 'heatmap';

  statsEl.textContent = `Running ${iters.toLocaleString()} iterations…`;
  // yield to browser to update UI, then run the heavy computation
  setTimeout(() => {
    game.run_simulation(iters);
    drawHeatmap(n);
    const prob = game.win_probability();
    const wins = game.wins();
    const losses = game.losses();
    statsEl.innerHTML =
      `Win probability (${iters.toLocaleString()} iterations): <strong>${prob.toFixed(10)}</strong><br>` +
      `Wins: ${wins.toLocaleString()}  Losses: ${losses.toLocaleString()}<br>` +
      game.move_stats();
    mode = 'idle';
  }, 20);
});

btnStop.addEventListener('click', () => {
  stopAnimation();
  statsEl.textContent = 'Stopped.';
});

// ── Initial state ─────────────────────────────────────────────────────────

{
  const n = 8;
  resizeCanvas(n);
  game = new WasmGame(n);
  drawBoard(n);
  statsEl.textContent = 'Press Animate or Heatmap to start.';
}
