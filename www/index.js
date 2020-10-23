import * as wasm from "wired-logic-rs";

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
    const buf = document.createElement("canvas").getContext("2d");
    buf.canvas.width = this.width;
    buf.canvas.height = this.height;

    let imageData = new ImageData(this.circuit.export(), this.width, this.height);
    buf.putImageData(imageData, 0, 0);
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

  ctx.clearRect(0, 0, circuit.width * scale, circuit.height * scale);
  ctx.drawImage(buf.canvas, 0, 0, circuit.width * scale, circuit.height * scale);

  ctx.beginPath();
  if (mouse.drag !== null) { // if mouse is being dragged
    const delta = { x: mouse.pos.x - mouse.drag.x, y: mouse.pos.y - mouse.drag.y };
    const startingPoint = { x: mouse.drag.x, y: mouse.drag.y };
    const endingPoint = {
      x: Math.abs(delta.x) > Math.abs(delta.y) ? mouse.pos.x : mouse.drag.x,
      y: Math.abs(delta.x) > Math.abs(delta.y) ? mouse.drag.y : mouse.pos.y,
    };

    // A line with inclusive ends:
    ctx.rect(startingPoint.x * scale, startingPoint.y * scale, scale, scale);

    ctx.rect(
      (startingPoint.x) * scale,
      (startingPoint.y) * scale,
      (endingPoint.x - startingPoint.x + 1) * scale,
      (endingPoint.y - startingPoint.y + 1) * scale
    );

    ctx.rect(endingPoint.x * scale, endingPoint.y * scale, scale, scale);
  } else if (mouse.hover) {
    // Cursor is visible. It could be either a rubber or a dot.
    if (rubber.on) {
      ctx.strokeRect(
        (mouse.pos.x - rubber.size/2 + 0.5)*scale,
        (mouse.pos.y - rubber.size/2 + 0.5)*scale,
        (rubber.size) * scale,
        (rubber.size) * scale
      );

      // Apply rubber
      circuit.circuit.fill_rect(
        mouse.pos.x-Math.floor(rubber.size/2),
        mouse.pos.y-Math.floor(rubber.size/2),
        rubber.size,
        rubber.size,
        wasm.Cell.Void,
      );
    } else {
      ctx.rect(mouse.pos.x * scale, mouse.pos.y * scale, scale, scale);
    }
  }

  // Draw hollow rects if the pixels are going to be toggled off.
  if (circuit.circuit.at((mouse.drag || mouse.pos).x, (mouse.drag || mouse.pos).y) == wasm.Cell.Wire)
    ctx.stroke();
  else ctx.fill();

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

  pauseButton.addEventListener("click", () => { setPause(!circuit.pause); }, false);

  document.getElementById("new-cancel").addEventListener("click", () => {
    document.getElementById("popup-container").style.display = "none";
  }, false);
  document.getElementById("new-proceed").addEventListener("click", () => {
    document.getElementById("popup-container").style.display = "none";

    const width = document.getElementById("new-width").value;
    const height = document.getElementById("new-height").value;

    if (isNaN(width) || width == 0 || isNaN(height) || height == 0) {
      alert("Wrong dimensions. Please try again.");
      return;
    }

    buf.canvas.width = width;
    buf.canvas.height = height;

    buf.fillStyle = "#000000";
    buf.fillRect(0, 0, width, height);
    buf.canvas.toBlob(blob => {
      new Response(blob).arrayBuffer().then(buffer => {
        initCircuit(new Uint8Array(buffer));
      });
    });
  }, false);

  document.getElementById("file-new").addEventListener("click", () => {
    document.getElementById("popup-container").style.display = "block";
  }, false);

  document.getElementById("file-open").addEventListener("click", () => {
    loadFile(initCircuit);
  }, false);

  document.getElementById("file-save").addEventListener("click", () => {
    downloadURL(circuit.exportDataURL(), "wired-export.png");
  }, false);

  document.getElementById("file-export").addEventListener("click", () => {
    function typedArrayToURL(typedArray, mimeType) {
      return URL.createObjectURL(new Blob([typedArray.buffer], {type: mimeType}));
    }

    if (confirm("This operation can take some time on large simulations.\n" +
                "Are you sure you want to proceed?")) {
      const array = circuit.circuit.render_gif(parseInt(circuit.delay));
      const url = typedArrayToURL(array, "image/gif");
      downloadURL(url, "wired-export.gif");
    }
  }, false);

  document.addEventListener("keydown", (e) => {
    switch (e.key) {
    case " ":
      setPause(!circuit.pause);
      break;
    case "+":
      setScale(scale+1);
      break;
    case "-":
      setScale(scale-1);
      break;
    case "x":
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
    case "x":
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

      // Toggle the line depending on the starting pixel
      let cell = wasm.Cell.Wire;
      if (circuit.circuit.at(mouse.drag.x, mouse.drag.y) == wasm.Cell.Wire)
        cell = wasm.Cell.Void;

      circuit.circuit.draw_line(mouse.drag.x, mouse.drag.y, endingPoint.x, endingPoint.y, cell);
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

let windowURL = new URL(window.location.href);
let url = windowURL.searchParams.get("url");

if (url == null) {
  url = DEFAULT_URL;
}

// Entry point: load the default example
loadURL(url, (bytes) => {
  initCircuit(bytes);
  initDocument();

  requestAnimationFrame(frame);
});
