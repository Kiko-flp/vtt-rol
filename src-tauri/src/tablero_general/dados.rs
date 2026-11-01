#[tauri::command]
pub fn tirar_dado() -> String {

    let tirada = rand::random_range(1..=20);

    // Por ahora devolvemos un texto simple desde Rust
    println!("¡El botón fue pulsado desde Svelte!");
    format!("¡Rust ha lanzado un d20 y ha salido un {tirada}!")
}