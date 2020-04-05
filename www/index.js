import * as wasm from "wirs";

const DEFAULT_SCALE = 1;
const DEFAULT_DELAY = 50;
const DEFAULT_PAUSE = false;
const DEFAULT_URL = "examples/7seg.png";

const ctx = document.getElementById("canvas").getContext("2d");
const buf = document.createElement("canvas").getContext("2d");

const delayRange = document.getElementById("delay-range");
const delayValue = document.getElementById("delay-value");
const pauseButton = document.getElementById("pause-button");

let circuit = null;
let scale = 1;

let mouse = {
  hover: false,
  pos: { x: 0, y: 0 },
  drag: null,
};

let rubber = {
  on: false,
  size: 1,
};

function Circuit(bytes) {
  this.circuit = wasm.Circuit.new(bytes);
  this.width = this.circuit.width();
  this.height = this.circuit.height();

  // As per The Living Standart, the ImageData does not perform a copy
  // when created with Uint8ClampedArray source.
  // pixel_view() returns an Uint8ClampedArray pointing at the region
  // of the wasm memory containing the rgb data; as a result, we save a few calls.
  this.pixels = new ImageData(this.circuit.pixels_view(), this.width, this.height);

  this.pause = false;
  this.delay = 0; // ms

  let timeoutID = null;
  let tick = () => {
    if (!this.pause) {
      this.circuit.tick();
    }

    timeoutID = setTimeout(tick, this.delay);
  };

  tick();

  this.exportDataURL = (type) => {
    this.circuit.reset();

    const buf = document.createElement("canvas").getContext("2d");
    buf.canvas.width = this.width;
    buf.canvas.height = this.height;

    buf.putImageData(this.pixels, 0, 0);
    return buf.canvas.toDataURL(type);
  };

  this.destroy = () => {
    clearTimeout(timeoutID);
    this.circuit.free();
  };
}

function setScale(v) {
  v = v > 0 ? v : 1;
  scale = v;
  initCanvas();
}

function setDelay(v) {
  circuit.delay = v >= 0 ? v : 0;
  delayRange.value = circuit.delay;
  delayValue.innerHTML = circuit.delay + " ms.";
}

function setPause(v) {
  circuit.pause = v;
  if (circuit.pause) {
    pauseButton.value = "play_arrow";
  } else {
    pauseButton.value = "pause";
  }
}

/* Animation frame */
function frame() {
  buf.putImageData(circuit.pixels, 0, 0);
  ctx.drawImage(buf.canvas, 0, 0, circuit.width * scale, circuit.height * scale);

  if (mouse.drag !== null) { // if mouse is being dragged
    const delta = { x: mouse.pos.x - mouse.drag.x, y: mouse.pos.y - mouse.drag.y };
    const startingPoint = { x: mouse.drag.x, y: mouse.drag.y };
    const endingPoint = {
      x: Math.abs(delta.x) > Math.abs(delta.y) ? mouse.pos.x : mouse.drag.x,
      y: Math.abs(delta.x) > Math.abs(delta.y) ? mouse.drag.y : mouse.pos.y,
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
  } else if (mouse.hover) {
    // Cursor is visible. It could be either a rubber or a dot.
    if (rubber.on) {
      ctx.beginPath();
      ctx.rect(
        (mouse.pos.x - rubber.size/2 + 0.5)*scale,
        (mouse.pos.y - rubber.size/2 + 0.5)*scale,
        (rubber.size) * scale,
        (rubber.size) * scale
      );
      ctx.stroke();

      // Apply rubber
      circuit.circuit.fill_rect(
        mouse.pos.x-Math.floor(rubber.size/2),
        mouse.pos.y-Math.floor(rubber.size/2),
        rubber.size,
        rubber.size,
        wasm.Cell.Void,
      );
    } else {
      ctx.fillRect(mouse.pos.x * scale, mouse.pos.y * scale, scale, scale);
    }
  }

  requestAnimationFrame(frame);
}

/* Sets the handlers and all necessary DOM interactions.
 * Is called only once. */
function initDocument() {
  function downloadURL(href, download) {
    var link = document.createElement('a');
    link.href = href;
    link.download = download;
    link.click();
  }

  delayRange.addEventListener("input", () => { setDelay(delayRange.value); }, false);
  document.getElementById("zoom-plus").addEventListener("click", () => { setScale(scale+1); }, false);
  document.getElementById("zoom-minus").addEventListener("click", () => { setScale(scale-1); }, false);

  pauseButton.addEventListener("click", () => { setPause(!pause); }, false);

  document.getElementById("file-open").addEventListener("click", () => {
    loadFile(initCircuit);
  }, false);

  document.getElementById("file-save").addEventListener("click", () => {
    downloadURL(circuit.exportDataURL(), "wired-export.png");
  }, false);

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
      rubber.on = true;
      mouse.drag = null;
      break;
    case 'z':
      if (mouse.hover) {
        circuit.circuit.fill_rect(mouse.pos.x, mouse.pos.y, 2, 2, wasm.Cell.Wire);
      }
      break;
    default:
      break;
    }
  });

  document.addEventListener("keyup", (e) => {
    switch (e.key) {
    case "Alt":
      rubber.on = false;
      break;
    default:
      break;
    }
  }, false);

  ctx.canvas.addEventListener("mouseover", () => { mouse.hover = true; }, false);
  ctx.canvas.addEventListener("mouseout", () => { mouse.hover = false; }, false);

  document.addEventListener("mousemove", (evt) => {
    function clamp(num, min, max) {
      return Math.min(Math.max(num, min), max);
    }

    var rect = ctx.canvas.getBoundingClientRect();
    mouse.pos = {
      x: clamp(Math.floor((evt.clientX - rect.left) / scale), 0, circuit.width-1),
      y: clamp(Math.floor((evt.clientY - rect.top) / scale), 0, circuit.height-1),
    };
  }, false);

  document.addEventListener("mousedown", () => {
    if (mouse.hover && !rubber.on) {
      mouse.drag = mouse.pos;
    }
  }, false);

  document.addEventListener("mouseup", () => {
    if (mouse.drag === null) {
      return;
    }

    if (mouse.hover && mouse.drag === mouse.pos) {
      circuit.circuit.toggle_pixel(mouse.pos.x, mouse.pos.y);
    } else {
      const delta = { x: mouse.pos.x - mouse.drag.x, y: mouse.pos.y - mouse.drag.y };
      const endingPoint = {
        x: Math.abs(delta.x) > Math.abs(delta.y) ? mouse.pos.x : mouse.drag.x,
        y: Math.abs(delta.x) > Math.abs(delta.y) ? mouse.drag.y : mouse.pos.y,
      };

      circuit.circuit.toggle_line(mouse.drag.x, mouse.drag.y, endingPoint.x, endingPoint.y);
    }

    mouse.drag = null;
  }, false);
}

/* After each scale change the canvas must be resized, and as a consequence all it's properties are lost.
 * This function performs resize as well as reinitialization of all canvas properties along with some
 * additional necessary logic. */
function initCanvas() {
  buf.canvas.width = circuit.width;
  buf.canvas.height = circuit.height;
  ctx.canvas.width = circuit.width * scale;
  ctx.canvas.height = circuit.height * scale;

  ctx.imageSmoothingEnabled = false;
  ctx.fillStyle = "white";
  ctx.strokeStyle = "white";

  switch (scale) {
  case 1:
    rubber.size = 12;
    break;
  case 2:
    rubber.size = 6;
    break;
  case 3:
    rubber.size = 3;
    break;
  case 4:
    rubber.size = 2;
    break;
  default:
    rubber.size = 1;
    break;
  }
}

/* Loads the circuit, sets default initial values. */
function initCircuit(bytes) {
  if (circuit)
    circuit.destroy();

  circuit = new Circuit(bytes);

  setPause(DEFAULT_PAUSE);
  setDelay(DEFAULT_DELAY);
  setScale(DEFAULT_SCALE);
}

function loadURL(filePath, callback) {
  var request = new XMLHttpRequest();
  request.open('GET', filePath, true);
  request.responseType = "arraybuffer";

  request.onreadystatechange = () => {
    if (request.response) {
      const bytes = new Uint8Array(request.response);

      callback(bytes);
    }
  };

  request.send(null);
}

function loadFile(callback) {
  const fileInput = document.createElement('input');
  fileInput.type="file";
  fileInput.addEventListener("change", (evt) => {
    const file = evt.target.files[0];

    let reader = new FileReader();
    reader.onload = (evt) => {
      const bytes = new Uint8Array(evt.target.result);

      callback(bytes);
    };

    reader.readAsArrayBuffer(file);
  }, false);

  fileInput.click();
}

// Entry point: load the default example
loadURL(DEFAULT_URL, (bytes) => {
  initCircuit(bytes);
  initDocument();

  requestAnimationFrame(frame);
});
