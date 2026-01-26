//! Widget CLI commands

use crate::windows::get_windows_dir;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the templates directory
fn get_templates_dir() -> PathBuf {
    // In development, use CARGO_MANIFEST_DIR
    // In production, templates should be in the app bundle
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
}

/// Get the builder script path
fn get_builder_script() -> Option<PathBuf> {
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("builder")
        .join("build.mjs");

    if dev_path.exists() {
        Some(dev_path)
    } else {
        None
    }
}

/// Create a new widget from template
pub fn create_widget(name: &str, template: &str) -> bool {
    let templates_dir = get_templates_dir();
    let template_path = templates_dir.join(template);

    if !template_path.exists() {
        eprintln!("Error: Template '{}' not found", template);
        eprintln!("Available templates:");
        if let Ok(entries) = fs::read_dir(&templates_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        eprintln!("  - {}", name);
                    }
                }
            }
        }
        return false;
    }

    let widgets_dir = match get_windows_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error: {}", e);
            return false;
        }
    };

    // Create widgets directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&widgets_dir) {
        eprintln!("Error creating widgets directory: {}", e);
        return false;
    }

    let widget_dir = widgets_dir.join(name);

    if widget_dir.exists() {
        eprintln!("Error: Widget '{}' already exists at {:?}", name, widget_dir);
        return false;
    }

    // Copy template to widget directory
    if let Err(e) = copy_dir_recursive(&template_path, &widget_dir) {
        eprintln!("Error copying template: {}", e);
        return false;
    }

    // Update widget.json with the widget name
    let widget_json_path = widget_dir.join("widget.json");
    if widget_json_path.exists() {
        if let Ok(content) = fs::read_to_string(&widget_json_path) {
            let updated = content
                .replace("{{WIDGET_ID}}", name)
                .replace("{{WIDGET_NAME}}", &capitalize(name));
            if let Err(e) = fs::write(&widget_json_path, updated) {
                eprintln!("Warning: Failed to update widget.json: {}", e);
            }
        }
    }

    println!("Created widget '{}' at {:?}", name, widget_dir);
    println!();
    println!("Next steps:");
    println!("  1. Edit {:?}", widget_dir.join("widget.json"));
    println!("  2. Edit your widget code");
    println!("  3. Restart Arcana to see your widget");

    true
}

/// Build a widget
pub fn build_widget(widget_id: &str) -> bool {
    let builder_script = match get_builder_script() {
        Some(path) => path,
        None => {
            eprintln!("Error: Builder script not found");
            return false;
        }
    };

    let widgets_dir = match get_windows_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error: {}", e);
            return false;
        }
    };

    if widget_id == "all" {
        // Build all widgets
        let mut success = true;
        if let Ok(entries) = fs::read_dir(&widgets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(id) = path.file_name().and_then(|n| n.to_str()) {
                        if has_buildable_sources(&path) {
                            if !build_single_widget(&builder_script, &path, id) {
                                success = false;
                            }
                        }
                    }
                }
            }
        }
        return success;
    }

    let widget_dir = widgets_dir.join(widget_id);
    if !widget_dir.exists() {
        eprintln!("Error: Widget '{}' not found", widget_id);
        return false;
    }

    build_single_widget(&builder_script, &widget_dir, widget_id)
}

/// List all widgets
pub fn list_widgets() -> bool {
    let widgets_dir = match get_windows_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error: {}", e);
            return false;
        }
    };

    if !widgets_dir.exists() {
        println!("No widgets directory found at {:?}", widgets_dir);
        return true;
    }

    let entries = match fs::read_dir(&widgets_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading widgets directory: {}", e);
            return false;
        }
    };

    println!("Widgets in {:?}:", widgets_dir);
    println!();

    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let widget_type = if has_buildable_sources(&path) {
                    "vue/react"
                } else if path.join("index.html").exists() {
                    "html"
                } else {
                    "unknown"
                };

                let built = if path.join(".arcana").join("index.html").exists() {
                    " (built)"
                } else {
                    ""
                };

                println!("  {} [{}]{}", name, widget_type, built);
                count += 1;
            }
        }
    }

    if count == 0 {
        println!("  (no widgets found)");
    }

    println!();
    println!("Total: {} widget(s)", count);

    true
}

// Helper functions

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

fn has_buildable_sources(widget_dir: &PathBuf) -> bool {
    if let Ok(entries) = fs::read_dir(widget_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "vue" || ext == "jsx" || ext == "tsx" {
                    return true;
                }
            }
        }
    }
    false
}

fn build_single_widget(builder_script: &PathBuf, widget_dir: &PathBuf, widget_id: &str) -> bool {
    println!("Building widget: {}", widget_id);

    let output = Command::new("node")
        .arg(builder_script)
        .arg("--widget")
        .arg(widget_dir)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("  Built successfully");
            true
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("  Build failed: {}", stderr);
            false
        }
        Err(e) => {
            eprintln!("  Failed to run builder: {}", e);
            false
        }
    }
}
