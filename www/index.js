import * as wasm from "wired-rs";

const ctx = document.getElementById("canvas").getContext('2d');
const buf = document.createElement("canvas").getContext('2d');

const delayRange = document.getElementById("delay-range");
const delayValue = document.getElementById("delay-value");
const pauseButton = document.getElementById("pause-button");

let imageDataView = null; // A slice view onto the pixels located in the wasm linear memory.

let circuit = null;
let width = 0;
let height = 0;

let pause; setPause(false);
let delay; setDelay(0);
let scale; setScale(1);

let lastTime = performance.now();

let mousePos = { x: 0, y: 0 }; // mouse position in simulation coordinates
let dragStart = null; // stores the position of the first click if dragging.

let hoveringCanvas = false;

let rubberOn = false;
var rubberSize = 10;

//
// Event handlers:
//

function setScale(v) {
  v = v > 0 ? v : 1;
  if (v != scale) {
    scale = v;
    initCanvas();
  }
}

function setDelay(v) {
  delay = v;
  delayRange.value = delay;
  delayValue.innerHTML = delay + " ms.";
}

function setPause(v) {
  pause = v;
  if (pause) {
    pauseButton.value = "play_arrow";
  } else {
    pauseButton.value = "pause";
  }
}

delayRange.addEventListener("input", () => { setDelay(delayRange.value); }, false);
document.getElementById("zoom-plus").addEventListener("click", () => { setScale(scale+1); }, false);
document.getElementById("zoom-minus").addEventListener("click", () => { setScale(scale-1); }, false);

pauseButton.addEventListener("click", () => { setPause(!pause); }, false);

document.addEventListener("keydown", (e) => {
  switch (e.key) {
  case " ":
    setPause(!pause);
    break;
  case "+":
    setScale(scale+1);
    break;
  case "-":
    setScale(scale-1);
    break;
  case "Alt":
    rubberOn = true;
    dragStart = null;
    break;
  case 'z':
    if (hoveringCanvas) {
      circuit.fill_rect(mousePos.x, mousePos.y, 2, 2, wasm.Cell.Wire);
    }
    break;
  default:
    break;
  }
});

document.addEventListener("keyup", (e) => {
  switch (e.key) {
  case "Alt":
    rubberOn = false;
    break;
  default:
    break;
  }
}, false);

// Canvas related event handlers:

canvas.addEventListener("mouseover", () => { hoveringCanvas = true; }, false);
canvas.addEventListener("mouseout", () => { hoveringCanvas = false; }, false);

document.addEventListener("mousemove", (evt) => {
  function clamp(num, min, max) {
    return Math.min(Math.max(num, min), max);
  }

  var rect = canvas.getBoundingClientRect();
  mousePos = {
    x: Math.floor((evt.clientX - rect.left) / scale),
    y: Math.floor((evt.clientY - rect.top) / scale),
  };

  mousePos = {
    x: clamp(mousePos.x, 0, width-1),
    y: clamp(mousePos.y, 0, height-1),
  };

  if (rubberOn) {
    circuit.fill_rect(
      mousePos.x-Math.floor(rubberSize/2),
      mousePos.y-Math.floor(rubberSize/2),
      rubberSize,
      rubberSize,
      wasm.Cell.Void,
    );
  }
}, false);

document.addEventListener("mousedown", () => {
  if (hoveringCanvas && !rubberOn) {
    dragStart = mousePos;
  }
}, false);

document.addEventListener("mouseup", () => {
  if (dragStart === null) {
    return;
  }

  if (hoveringCanvas && dragStart === mousePos) {
    circuit.toggle_pixel(mousePos.x, mousePos.y);
  } else {
    const delta = { x: mousePos.x - dragStart.x, y: mousePos.y - dragStart.y };
    const endingPoint = {
      x: Math.abs(delta.x) > Math.abs(delta.y) ? mousePos.x : dragStart.x,
      y: Math.abs(delta.x) > Math.abs(delta.y) ? dragStart.y : mousePos.y,
    };

    circuit.toggle_line(dragStart.x, dragStart.y, endingPoint.x, endingPoint.y);
  }

  dragStart = null;
}, false);

//
// Animation:
//

function loop() {
  let now = performance.now();
  if (!pause && now - lastTime > delay) {
    circuit.tick();

    lastTime = now;
  }

  buf.putImageData(imageDataView, 0, 0);
  ctx.drawImage(buf.canvas, 0, 0, width, height, 0, 0, width * scale, height * scale);

  if (dragStart !== null) { // if mouse is being dragged
    const delta = { x: mousePos.x - dragStart.x, y: mousePos.y - dragStart.y };
    const startingPoint = { x: dragStart.x, y: dragStart.y };
    const endingPoint = {
      x: Math.abs(delta.x) > Math.abs(delta.y) ? mousePos.x : dragStart.x,
      y: Math.abs(delta.x) > Math.abs(delta.y) ? dragStart.y : mousePos.y,
    };

    // A line with inclusive ends:
    ctx.fillRect(startingPoint.x * scale, startingPoint.y * scale, scale, scale);

    ctx.fillRect(
      (startingPoint.x) * scale,
      (startingPoint.y) * scale,
      (endingPoint.x - startingPoint.x + 1) * scale,
      (endingPoint.y - startingPoint.y + 1) * scale
    );

    ctx.fillRect(endingPoint.x * scale, endingPoint.y * scale, scale, scale);
  } else if (hoveringCanvas) {
    // Cursor is visible. It could be either a rubber or a dot.
    if (rubberOn) {
      ctx.beginPath();
      ctx.rect(
        (mousePos.x - rubberSize/2 + 0.5)*scale,
        (mousePos.y - rubberSize/2 + 0.5)*scale,
        (rubberSize) * scale,
        (rubberSize) * scale
      );
      ctx.stroke();
    } else {
      ctx.fillRect(mousePos.x * scale, mousePos.y * scale, scale, scale);
    }
  }


  requestAnimationFrame(loop);
}

function initCanvas() {
  ctx.canvas.width = width * scale;
  ctx.canvas.height = height * scale;

  ctx.imageSmoothingEnabled = false;
  ctx.fillStyle = "white";
  ctx.strokeStyle = "white";

  switch (scale) {
  case 1:
    rubberSize = 12;
    break;
  case 2:
    rubberSize = 6;
    break;
  case 3:
    rubberSize = 3;
    break;
  default:
    rubberSize = 1;
    break;
  }
}

function loadCircuit(filePath) {
  var request = new XMLHttpRequest();
  request.open('GET', filePath, true);
  request.responseType = "arraybuffer";

  request.onreadystatechange = function() {
    if (request.response) {
      const bytes = new Uint8Array(request.response);
      circuit = wasm.Circuit.new(bytes);
      width = circuit.width();
      height = circuit.height();

      buf.canvas.width = width;
      buf.canvas.height = height;

      // As per The Living Standart, the ImageData does not perform a copy
      // when created with Uint8ClampedArray source.
      // pixel_view() returns an Uint8ClampedArray pointing at the region
      // of the wasm memory containing the rgb data; as a result, we save a few calls.
      imageDataView = new ImageData(circuit.pixels_view(), width);

      initCanvas();
      loop();
    }
  };

  request.send(null);
}

loadCircuit("examples/input.gif");
