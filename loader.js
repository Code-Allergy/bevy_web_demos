let currentModuleIndex = 0; // Track the current module index
// let wasm_modules = ['./binds/assignment_1/bind.js', ]; // List of WASM modules
let wasm_modules = []; // List of WASM modules
fetch('./modules.txt')
    .then(response => {
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return response.text();
    })
    .then(modulesText => {
        // Split the text into an array, removing any empty lines
        wasm_modules = modulesText.split('\n').filter(module => module.trim() !== '');
        console.log("Loaded WASM modules:", wasm_modules);

        // Load the first WASM module
        loadCurrentModule();
    })
    .catch(error => {
        console.error("Failed to load WASM modules:", error);
    });

async function loadWasmModule(modulePath) {
    const { default: init, demo_name } = await import(modulePath);

    // Initialize the module
    await init().catch((error) => {
      if (!error.message.startsWith("Using exceptions for control flow, don't mind me. This isn't actually an error!")) {
        throw error;
      }
    });

    // get the title and place the demo name in the title
    document.getElementById('demo_title').innerText = demo_name();

    console.log(demo_name());
    console.log(`${modulePath} loaded`);
}

// Function to remove the current canvas and spawn a new one
function resetCanvas() {
// Remove the first canvas found on the page
    // const oldCanvas = document.querySelector('canvas');
    // if (oldCanvas) {
    //     oldCanvas.remove(); // Remove the canvas element
    // }

    // // Create a new canvas element
    // const canvas = document.createElement('canvas');
    // canvas.width = 800;
    // canvas.height = 600;
    // canvas.id = 'demo_canvas';
    // document.body.appendChild(canvas); // Append the canvas to the body
}

// Function to load the current module based on the index
async function loadCurrentModule() {
    resetCanvas(); // Reset the canvas
    await loadWasmModule(wasm_modules[currentModuleIndex]); // Load the current module
}


// Event listener for the forward button
document.getElementById('forward').addEventListener('click', () => {
    currentModuleIndex = (currentModuleIndex + 1) % wasm_modules.length;
    loadCurrentModule(); // Load the current module
});
  
// Event listener for the backward button
document.getElementById('backward').addEventListener('click', () => {
    currentModuleIndex = (currentModuleIndex - 1 + wasm_modules.length) % wasm_modules.length;
    loadCurrentModule(); // Load the current module
});