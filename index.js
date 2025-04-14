import init, * as wasm from './pkg/your_project_name.js';

async function run() {
    // Initialize the wasm module
    await init();

    // Get user-defined rows and cols, or hardcode for now
    const rows = 999;
    const cols = 18248;

    // Call the new `init` exported from Rust
    wasm.init(rows, cols);

    // Now safely render spreadsheet
    const spreadsheetHTML = wasm.render_initial_spreadsheet();
    document.getElementById("spreadsheet").innerHTML = spreadsheetHTML;

    // Example formula update
    document.getElementById("submit").addEventListener("click", async () => {
        const input = document.getElementById("formula").value;
        try {
            const updatedHTML = await wasm.update_formula(input);
            document.getElementById("spreadsheet").innerHTML = updatedHTML;
        } catch (err) {
            console.error("Formula update error:", err);
            alert("Error updating formula: " + err);
        }
    });
    
}

run();
