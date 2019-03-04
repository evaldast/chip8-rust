let pixelArray;
let canvas;
let context;
let screen;
const width = 64;
const height = 32;

initializeCanvas();

fetchAndInstantiateWasm('./chip8_bg.wasm')
  .then(m => {
    const ptr = m.next_frame();
    pixelArray = new Uint8Array(m.memory.buffer, ptr, 2048);
  })

main();

function main() {
  window.requestAnimationFrame(main);

  createImage();
}

function createImage() {
  if (!pixelArray) {
    return;
  }

  for (var x = 0; x < pixelArray.length; x++) {
    var alphaIndex = x * 4 + 3;

    if (pixelArray[x] == 1) {
      screen.data[alphaIndex] = 0;
    }
  }

  context.putImageData(screen, 0, 0);
}

function fetchAndInstantiateWasm(url, imports) {
  return fetch(url)
    .then(res => {
      if (res.ok)
        return res.arrayBuffer();
      throw new Error(`Unable to fetch Web Assembly file ${url}.`);
    })
    .then(bytes => WebAssembly.compile(bytes))
    .then(module => WebAssembly.instantiate(module, imports || {}))
    .then(instance => instance.exports);
}

function initializeCanvas() {
  canvas = window.document.getElementById('canvas');
  context = canvas.getContext('2d');
  canvas.width = width;
  canvas.height = height;

  document.body.appendChild(canvas);
  
  screen = context.createImageData(width, height);
  screen.data.fill(0, -1, -1);
}