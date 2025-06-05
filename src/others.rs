use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::fs;
use std::path::{Path, PathBuf};

/// Recursively finds all `.py` files under `dir` and executes their code
/// in the `__main__` namespace of the interpreter.
///
/// In practice this means: for every .py file, we read its contents into a string,
/// then do `py.run(code, None, Some(main.dict()))`, so that everything those
/// files define (classes, functions, globals) gets placed into __main__.
fn load_python_files(py: Python<'_>, dir: &Path) -> PyResult<()> {
    // Grab a reference to __main__ and its globals dict
    let main_mod = py.import("__main__")?;
    let globals = main_mod.dict();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            load_python_files(py, &path)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("py") {
            // Read the .py file to a string
            let code = fs::read_to_string(&path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyOSError, _>(format!("{e}")))?;

            // Execute it in __main__’s namespace
            //
            // This is essentially:
            //    >>> exec(code, __main__.__dict__)
            //
            py.run(&code, None, Some(globals))?;
        }
    }

    Ok(())
}

fn do_a_thing() -> PyResult<()> {
    // Expect exactly one CLI argument: the path to the “plugins” directory.
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <plugins_directory>", args[0]);
        std::process::exit(1);
    }
    let plugins_dir = PathBuf::from(&args[1]);
    if !plugins_dir.is_dir() {
        eprintln!("Error: '{}' is not a directory.", plugins_dir.display());
        std::process::exit(1);
    }

    // Acquire the GIL and run everything inside Python::with_gil
    Python::with_gil(|py| {
        // 1) Optionally insert the plugins_dir at sys.path[0],
        //    so that any `import` inside those .py files can
        //    do relative imports or siblings.
        let sys_path: &PyList = py.import("sys")?.getattr("path")?.downcast()?;
        sys_path.insert(0, plugins_dir.to_str().unwrap())?;

        // 2) Recursively load & exec every .py file under plugins_dir into __main__
        load_python_files(py, &plugins_dir)?;

        // 3) Now assume that one of those modules defined:
        //       def get_all_plugins():
        //           return [MyPluginClass1, MyPluginClass2, …]
        //
        //    We fetch that function out of __main__ and call it.
        let main_mod = py.import("__main__")?;
        let get_all_plugins = main_mod.getattr("get_all_plugins")?;
        let plugin_list_obj = get_all_plugins.call0()?; // should return a Python list

        // Downcast to a PyList to iterate
        let plugin_list: &PyList = plugin_list_obj.downcast()
            .map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "get_all_plugins() did not return a list"
            ))?;

        // 4) Iterate over each “class” in that list
        for plugin in plugin_list.iter() {
            // Attempt to grab plugin.__dict__ so we can print attribute names & values
            match plugin.getattr("__dict__") {
                Ok(dict_obj) => {
                    let attr_dict: &PyDict = dict_obj.downcast()?;
                    println!("Plugin class object = {:?}", plugin);
                    for (py_key, py_val) in attr_dict {
                        // We convert key and val to strings for printing
                        let key_str: String = py_key.str()?.to_str()?.to_owned();
                        let val_str: String = py_val.repr()?.to_str()?.to_owned();
                        println!("  {} => {}", key_str, val_str);
                    }
                }
                Err(_) => {
                    // If it has no __dict__, just print the repr
                    println!("Plugin (no __dict__): {:?}", plugin);
                }
            }
        }

        Ok(())
    })
}
