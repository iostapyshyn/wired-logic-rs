import * as wasm from "wired-rs";

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext('2d');

let app;
let width;
let height;

let pause;
let delay;
let scale;

let lastTime = performance.now();

const pauseButton = document.getElementById("pauseButton");
const scaleRange = document.getElementById("scaleRange");
const speedRange = document.getElementById("speedRange");

//
// Event handlers:
//

function setPause(v) {
  pause = v;
  if (pause) {
    pauseButton.value = "play_arrow";
  } else {
    pauseButton.value = "pause";
  }
}

function setScale(v) {
  scale = v;
  initCanvas();
}

// Ensure consistent range input behaviour on all browsers.
function onRangeChange(rangeInputElmt, listener) {
  var inputEvtHasNeverFired = true;
  var rangeValue = {current: undefined, mostRecent: undefined};

  rangeInputElmt.addEventListener("input", function(evt) {
    inputEvtHasNeverFired = false;
    rangeValue.current = evt.target.value;
    if (rangeValue.current !== rangeValue.mostRecent) {
      listener(evt);
    }
    rangeValue.mostRecent = rangeValue.current;
  });

  rangeInputElmt.addEventListener("change", function(evt) {
    if (inputEvtHasNeverFired) {
      listener(evt);
    }
  });
}

onRangeChange(scaleRange, function (e) {
  setScale(e.target.value);
});

onRangeChange(speedRange, function (e) {
  delay = e.target.value;
});

document.addEventListener('keypress', (event) => {
  const keyName = event.key;

  if (keyName === ' ') {
    setPause(!pause);
  }
});

pauseButton.addEventListener("click", function () { setPause(!pause); });

//
// Animation:
//

function renderImageData() {
  const pixels = app.render();
  return new ImageData(pixels, width);
}

function animationFrame() {
  let now = performance.now();
  if (!pause && now - lastTime > delay) {
    app.tick();

    ctx.putImageData(renderImageData(), 0, 0);
    ctx.drawImage(canvas, 0, 0, width, height, 0, 0, width * scale, height * scale);

    lastTime = now;
  }

  requestAnimationFrame(animationFrame);
}

function initCanvas() {
  canvas.width = width * scale;
  canvas.height = height * scale;

  ctx.imageSmoothingEnabled = false;
}

function loadCircuit(filePath) {
  var request = new XMLHttpRequest();
  request.open('GET', filePath, true);
  request.responseType = "arraybuffer";

  request.onreadystatechange = function() {
    if (request.response) {
      const bytes = new Uint8Array(request.response);

      app = wasm.App.new(bytes);
      width = app.width();
      height = app.height();

      initCanvas();
      animationFrame();
    }
  };

  request.send(null);
}

setPause(false);
setScale(scaleRange.value);

delay = speedRange.value;

loadCircuit("examples/input.gif");
