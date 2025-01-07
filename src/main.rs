use leptos::*;
use wasm_bindgen::prelude::*;
use web_sys::{FileReader, File, ProgressEvent, HtmlInputElement};

#[wasm_bindgen]
pub fn read_file(file: File) -> Result<String, JsValue> {
    let reader = FileReader::new()?;
    
    reader.read_as_text(&file)?;

    reader.set_onloadend(Some(|event: ProgressEvent| {
        let target = event.target().unwrap();
        let reader = target.dyn_ref::<FileReader>().unwrap();  // Cast target to FileReader
        let file_content = reader.result().unwrap().as_string().unwrap();
        display_content(file_content);  // Call the function to display the content
    }));

    Ok("File read successfully".to_string())  // Temporary response
}

// Function to display the file content (to be updated for WordRunner simulation)
fn display_content(content: String) {
    println!("{}", content);
}

// Leptos component to handle file input and display
#[component]
pub fn file_upload(cx: Scope) -> impl IntoView {
    // Properly return RSX inside double curly braces
    view! { cx,
        <div>
            <h1>"Upload a file for RSVP Simulation!"</h1>
            <input type="file" id="fileInput" onchange=|ev: web_sys::Event| {
                let file_input = ev.target()
                    .expect("Failed to get event target")
                    .dyn_into::<HtmlInputElement>()
                    .expect("Failed to cast to HtmlInputElement");

                if let Some(files) = file_input.files() {
                    if let Some(file) = files.get(0) {
                        // Call the file read function
                        read_file(file);
                    }
                }
            } as Box<dyn IntoAttribute> />
            <div id="fileContent">"File content will appear here..."</div>
        </div>
    }
}

// Main function to launch the app
fn main() {
    leptos::mount_to_body::<file_upload, _>(file_upload);  // Correct number of generic arguments
}
