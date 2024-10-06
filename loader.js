let currentModuleIndex = 0;
let wasm_modules = [];
let wasmContext;

function getModuleIndexFromURL() {
    const urlParams = new URLSearchParams(window.location.search);
    const index = parseInt(urlParams.get('module'));
    return isNaN(index) ? 0 : index;
}

function updateURLWithModuleIndex(index) {
    const url = new URL(window.location);
    url.searchParams.set('module', index);
    window.history.pushState({}, '', url);
}

fetch('./modules.txt')
    .then(response => {
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return response.text();
    })
    .then(modulesText => {
        wasm_modules = modulesText.split('\n').filter(module => module.trim() !== '');
        console.log("Loaded WASM modules:", wasm_modules);

        currentModuleIndex = getModuleIndexFromURL();
        if (currentModuleIndex >= wasm_modules.length) {
            currentModuleIndex = 0;
        }
        loadCurrentModule();
    })
    .catch(error => {
        console.error("Failed to load WASM modules:", error);
    });

async function loadWasmModule(modulePath) {
    try {
        if (wasmContext) {
            await destroy();
            document.getElementById('demo_title').innerText = "Loading...";
        }

        const module = await import(modulePath);
        await module.default();
        wasmContext = module;

        document.getElementById('demo_title').innerText = wasmContext.demoName();


        console.log(`${modulePath} loaded`);
        const sourceFile = wasmContext.sourceFile();
        if (sourceFile) {
            const code_element = document.getElementById("demo_code");
            code_element.textContent = sourceFile.trim();
            await Prism.highlightAll();
        } else {
            document.getElementById("demo_code").innerText = "No source code available";
        }

        wasmContext.startGame();
    } catch (error) {
        console.error(`Failed to load module ${modulePath}:`, error);
    }
}

function resetCanvas() {
    const oldCanvas = document.querySelector('canvas');
    if (oldCanvas) {
        oldCanvas.remove();
    }
}

async function loadCurrentModule() {
    updateURLWithModuleIndex(currentModuleIndex);
    await loadWasmModule(wasm_modules[currentModuleIndex]);
}

document.getElementById('forward').addEventListener('click', () => {
    currentModuleIndex = (currentModuleIndex + 1) % wasm_modules.length;
    loadCurrentModule();
});

document.getElementById('backward').addEventListener('click', () => {
    currentModuleIndex = (currentModuleIndex - 1 + wasm_modules.length) % wasm_modules.length;
    loadCurrentModule();
});

async function destroy() {
    if (wasmContext?.stopGame) {
        try {
            await wasmContext.stopGame();
        } catch (error) {
            console.error("Error stopping game:", error);
        }
    }
    wasmContext = undefined;
}

window.addEventListener('beforeunload', destroy);

// Add this to handle browser back/forward navigation
window.addEventListener('popstate', () => {
    currentModuleIndex = getModuleIndexFromURL();
    loadCurrentModule();
});