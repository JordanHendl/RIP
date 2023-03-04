use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
  if dir.is_dir() {
      for entry in fs::read_dir(dir)? {
          let entry = entry?;
          let path = entry.path();
          if path.is_dir() {
              visit_dirs(&path, cb)?;
          } else {
              cb(&entry);
          }
      }
  }
  Ok(())
}

fn timestamps_differ(first: &Path, second: &Path) -> bool {
  let meta_one = std::fs::metadata(first);
  let meta_two = std::fs::metadata(second);

  if first.exists() && second.exists() {
    let mod_one = meta_one.unwrap().modified().unwrap();
    let mod_two = meta_two.unwrap().modified().unwrap();
    
    return mod_one != mod_two;
  }

  return true;
}

fn compile_shaders() {
  // Make output directory for shaders.
  fs::create_dir_all("./target/shaders").expect("Failed to create shader compilation output directory!");
  let path = Path::new("./shaders");
  let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

  visit_dirs(path, & mut |dir_entry|
  {
    let mut string : String = "cargo:rerun-if-changed=".to_owned();
    let path = dir_entry.path().clone();
    let file_name = dir_entry.file_name().to_owned();
    let tmp_stem = std::path::Path::new(path.file_stem().clone().unwrap());
    let stem = tmp_stem.file_stem().clone();
  
    // Make sure we rebuild on change.
    string.push_str(file_name.clone().to_str().unwrap());
    println!("{}", string);
  
    let final_output_path = out_path.join("shaders.rs"); 
    if final_output_path.exists() || timestamps_differ(&path, &final_output_path) {
      // Now, we try to compile it.
      let mut shader_full_path = std::string::String::from("./shaders");
      shader_full_path.push_str("/");
      shader_full_path.push_str(&file_name.to_str().unwrap());
      
      let output = format!("./target/shaders/{stem}.spirv", stem=stem.unwrap().to_str().unwrap());
      
      let status = std::process::Command::new("glslangValidator")
      .args(["-g", "-V", "-Od", "-o", &output, &shader_full_path])
      .status()
      .expect("Failed to compile shader!");
      assert!(status.success());
    }
  }).expect("Failed to recursively parse directory!");
}


fn main() {
  compile_shaders();
}
